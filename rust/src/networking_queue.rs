/*
 * gRIP
 * Copyright (c) 2018 Alik Aslanyan <cplusplus256@gmail.com>
 *
 *
 *    This program is free software; you can redistribute it and/or modify it
 *    under the terms of the GNU General Public License as published by the
 *    Free Software Foundation; either version 3 of the License, or (at
 *    your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful, but
 *    WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *    General Public License for more details.
 *
 *    You should have received a copy of the GNU General Public License
 *    along with this program; if not, write to the Free Software Foundation,
 *    Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 *    In addition, as a special exception, the author gives permission to
 *    link the code of this program with the Half-Life Game Engine ("HL
 *    Engine") and Modified Game Libraries ("MODs") developed by Valve,
 *    L.L.C ("Valve").  You must obey the GNU General Public License in all
 *    respects for all of the code used other than the HL Engine and MODs
 *    from Valve.  If you modify this file, you may extend this exception
 *    to your version of the file, but you are not obligated to do so.  If
 *    you do not wish to do so, delete this exception statement from your
 *    version.
 *
 */

use std::thread;

use futures::future;
use futures::prelude::*;
use futures::prelude::*;
use hyper::rt::*;
use std::mem;
use std::time::{Duration, Instant};

use crate::errors::*;
use std::cell::RefCell;

use either::*;

#[derive(Clone, Debug)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Builder, Constructor, Clone, Debug)]
pub struct Request {
    pub http_type: RequestType,
    pub uri: hyper::Uri,
    pub body: Vec<u8>,
}

#[derive(Constructor, Builder)]
pub struct Response {
    pub base_request: Request,
    pub body: Vec<u8>,
}

// TODO: Replace with trait alias, when they became stable
// https://github.com/rust-lang/rust/issues/41517
type ResponseCallBack = Fn(Result<Response>) + Sync + Send;

enum InputCommand {
    Request {
        request: Request,
        callback: Box<ResponseCallBack>,
    },
    Quit,
}

enum OutputCommand {
    Response {
        response: Response,
        callback: Box<ResponseCallBack>,
    },
    Error {
        error: Error,
        callback: Box<ResponseCallBack>,
    },
}

pub struct Queue {
    working_thread: Option<thread::JoinHandle<()>>,
    executor: tokio::runtime::TaskExecutor,
    input_command_sender: futures::sync::mpsc::UnboundedSender<InputCommand>,
    response_receiver: crossbeam_channel::Receiver<OutputCommand>,
    last_time_executed_with_limit: Option<Instant>,
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Queue {
    pub fn new(number_of_dns_threads: usize) -> Self {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let executor = runtime.executor();
        let (input_command_sender, input_command_receiver) = futures::sync::mpsc::unbounded();
        let (response_sender, response_receiver) = crossbeam_channel::unbounded();

        let client = {
            let https = hyper_tls::HttpsConnector::new(number_of_dns_threads);
            crate::client::Client::new(
                hyper::Client::builder()
                    .executor(executor.clone())
                    .build::<_, hyper::Body>(https.unwrap()),
            )
        };

        let working_thread = {
            let executor = executor.clone();
            clone_all!(response_sender);
            thread::spawn(move || {
                clone_all!(response_sender);
                runtime
                    .block_on(lazy(move || {
                        clone_all!(response_sender);
                        input_command_receiver
                            .take_while(|cmd| {
                                Ok(match cmd {
                                    InputCommand::Quit => {
                                        info!("Received quit command. New commands will not be received");
                                        false
                                    },
                                    _ => true,
                                })
                            }).for_each(move |cmd| {
                                clone_all!(response_sender);
                                match cmd {
                                    InputCommand::Quit => unreachable!(),
                                    InputCommand::Request { request, callback } => {

                                        executor.spawn(
                                            client.request(match request.http_type {
                                                RequestType::Post => hyper::Request::post(request.uri.clone()),
                                                RequestType::Get => hyper::Request::get(request.uri.clone()),
                                                RequestType::Delete => hyper::Request::delete(request.uri.clone()),
                                                RequestType::Put => hyper::Request::put(request.uri.clone()),
                                            }.body(hyper::Body::from(request.body.clone())).unwrap()) // TODO: Optimize clone away
                                                .and_then(move |res| res.into_body().concat2())
                                                .map(|body| {
                                                    use bytes::buf::FromBuf;
                                                    Either::Left((request, Vec::from_buf(body.into_bytes())))
                                                })
                                                .or_else(|e| {
                                                    future::ok(Either::Right(ErrorKind::HTTPError(e).into()))
                                                })
                                                .and_then(move |either| {
                                                    future::ok(match either {
                                                        Either::Left((request, vec)) => {
                                                            response_sender.send(OutputCommand::Response {
                                                                response: Response::new(
                                                                    request,
                                                                    vec,
                                                                ),
                                                                callback
                                                            }).unwrap()
                                                        },
                                                        Either::Right(error) => {
                                                            response_sender.send(OutputCommand::Error {
                                                                error,
                                                                callback,
                                                            }).unwrap();
                                                        }
                                                    })
                                                }).map(|_| {})
                                        )
                                    }
                                }

                                Ok(())
                            })
                    })).unwrap();
            })
        };

        Queue {
            working_thread: Some(working_thread),
            executor,
            input_command_sender,
            response_receiver,
            last_time_executed_with_limit: None,
        }
    }

    pub fn stop(&mut self) {
        // TODO: Make other functions report error when queue was stopped
        self.send_input_command(InputCommand::Quit);
        if let Some(thread) = mem::replace(&mut self.working_thread, None) {
            thread.join().unwrap();
        }
    }

    pub fn send_request<T: 'static + Fn(Result<Response>) + Sync + Send>(
        &mut self,
        request: Request,
        callback: T,
    ) {
        self.send_input_command(InputCommand::Request {
            request,
            callback: Box::new(callback),
        });
    }

    fn send_input_command(&mut self, input_command: InputCommand) {
        let input_command_sender = self.input_command_sender.clone();
        self.executor.spawn(lazy(move || {
            input_command_sender
                .send(input_command)
                .map(|_| {})
                .map_err(|_| {})
        }));
    }

    fn try_recv_queue(&mut self) -> Result<()> {
        match self.response_receiver.try_recv()? {
            OutputCommand::Response { response, callback } => {
                (callback)(Ok(response));
            }
            OutputCommand::Error { error, callback } => {
                (callback)(Err(error));
            }
        }

        Ok(())
    }

    pub fn execute_queue_with_limit(
        &mut self,
        limit: usize,
        delay_between_executions: Duration,
    ) -> usize {
        if let Some(last_time) = self.last_time_executed_with_limit {
            if Instant::now().duration_since(last_time) <= delay_between_executions {
                return 0;
            }
        }

        self.last_time_executed_with_limit = Some(Instant::now());

        let mut counter = 0;
        while counter <= limit {
            if self.try_recv_queue().is_err() {
                break;
            }
            counter += 1;
        }
        counter
    }

    pub fn execute_query_with_timeout(&mut self, timeout: Duration, one_step_timeout: Duration) {
        let instant = Instant::now();

        while Instant::now().duration_since(instant) <= timeout {
            self.try_recv_queue().ok();
            thread::sleep(one_step_timeout);
        }
    }
}

mod tests {
    #[test]
    fn test() {
        use super::*;
        use std::sync::{Arc, Mutex};

        let mut queue = Queue::new(4);

        use std::default::Default;

        let control_variable = Arc::new(Mutex::new(false));
        let control_variable_c = Arc::clone(&control_variable);
        queue.send_request(
            RequestBuilder::default()
                .http_type(RequestType::Get)
                .body(vec![])
                .uri("https://docs.rs/".parse().unwrap())
                .build()
                .unwrap(),
            move |req| {
                *control_variable_c.lock().unwrap() = true;
                assert!(String::from_utf8_lossy(&req.unwrap().body[..]).contains("docs.rs"));
            },
        );

        assert_eq!(*control_variable.lock().unwrap(), false);

        queue.execute_query_with_timeout(Duration::from_secs(5), Duration::from_millis(100));

        assert_eq!(*control_variable.lock().unwrap(), true);
    }
}
