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
use futures::sync::oneshot;
use std::mem;
use std::time::{Duration, Instant};

use crate::errors::*;

use tokio::prelude::FutureExt;

use reqwest::r#async as reqwest_async;

#[derive(Clone, Debug)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug)]
pub struct RequestCancellation(oneshot::Sender<()>);

#[derive(Constructor, Builder, Clone, Debug, Default)]
pub struct RequestOptions {
    #[builder(default)]
    pub headers: reqwest::header::HeaderMap,

    #[builder(default)]
    pub timeout: Option<Duration>,
}

#[derive(Builder, Clone, Constructor, Debug)]
pub struct Request {
    pub http_type: RequestType,
    pub uri: reqwest::Url,

    #[builder(default)]
    pub body: Vec<u8>,

    #[builder(default)]
    pub options: RequestOptions,
}

#[derive(Constructor, Builder)]
pub struct Response {
    pub base_request: Request,
    pub body: Vec<u8>,
    pub status_code: reqwest::StatusCode,
}

// TODO: Replace with trait alias, when they became stable
// https://github.com/rust-lang/rust/issues/41517
type ResponseCallBack = Fn(Result<Response>) + Sync + Send;

enum InputCommand {
    Request {
        cancellation_signal: oneshot::Receiver<()>,
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
    number_of_pending_requests: usize,
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Queue {
    pub fn new() -> Self {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let executor = runtime.executor();

        let (input_command_sender, input_command_receiver) = futures::sync::mpsc::unbounded();
        let (response_sender, response_receiver) = crossbeam_channel::unbounded();

        let client = reqwest_async::Client::new();

        let working_thread = {
            let executor = executor.clone();
            clone_all!(response_sender);
            thread::spawn(move || {
                clone_all!(response_sender);
                runtime
                    .block_on(future::lazy(move || {
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
                                    InputCommand::Request { request, callback, cancellation_signal } => {

                                        enum State {
                                            Successful(Vec<u8>, reqwest::StatusCode),
                                            Error(Error),
                                            Canceled,
                                            Timeout
                                        }


                                        executor.spawn(
                                            // Request construction.
                                            match request.http_type {
                                                RequestType::Post => client.post(request.uri.clone()),
                                                RequestType::Get => client.get(request.uri.clone()),
                                                RequestType::Delete => client.delete(request.uri.clone()),
                                                RequestType::Put => client.put(request.uri.clone()),
                                            }
                                                .body(reqwest_async::Body::from(request.body.clone()))
                                                .headers(request.options.headers.clone()) // TODO: Optimize clone away
                                                .send()
                                                .and_then(move |res| {
                                                    let status = res.status();
                                                    res.into_body().concat2().map(move |body| (status, body))
                                                })
                                                // Cancelling / Error handling.
                                                .map(|(status_code, body)| {
                                                    State::Successful(body.to_vec(), status_code)
                                                })
                                                .or_else(|e| {
                                                    future::ok(State::Error(ErrorKind::HTTPError(e).into()))
                                                })
                                                .select2(cancellation_signal
                                                    .map(|_| State::Canceled)
                                                    .or_else(|_| future::ok(State::Canceled))
                                                )
                                                .map_err(|_: future::Either<((), _), ((), _)>| unreachable!())
                                                .map(|either| {
                                                    either.split().0
                                                })
                                                // Timeout.
                                                .timeout(request.options.timeout.clone()
                                                    .unwrap_or_else(|| Duration::new(std::u16::MAX as u64, 0)))
                                                .or_else(|_| future::ok(State::Timeout))
                                                .map_err(|_:tokio::timer::Error| unreachable!())
                                                // Sending output command.
                                                .and_then(move |state| {
                                                    match state {
                                                        State::Successful(vec, status_code) => {
                                                            response_sender.send(OutputCommand::Response {
                                                                response: Response::new(
                                                                    request,
                                                                    vec,
                                                                    status_code
                                                                ),
                                                                callback
                                                            }).unwrap()
                                                        },
                                                        State::Error(error) => {
                                                            response_sender.send(OutputCommand::Error {
                                                                error,
                                                                callback,
                                                            }).unwrap();
                                                        },
                                                        State::Canceled => {
                                                            response_sender.send(OutputCommand::Error {
                                                                error: ErrorKind::RequestCancelled.into(),
                                                                callback,
                                                            }).unwrap();
                                                        }
                                                        State::Timeout => {
                                                            response_sender.send(OutputCommand::Error {
                                                                error: ErrorKind::RequestTimeout.into(),
                                                                callback,
                                                            }).unwrap()
                                                        }
                                                    }
                                                    future::ok(())
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
            number_of_pending_requests: 0,
        }
    }

    pub fn stop(&mut self) {
        // TODO: Make other functions report error when queue was stopped
        self.send_input_command(InputCommand::Quit);
        if let Some(thread) = mem::replace(&mut self.working_thread, None) {
            thread.join().unwrap();
        }
    }

    #[must_use = "this `RequestCancellation` should be alive, because when it drops request cancels."]
    pub fn send_request<T: 'static + Fn(Result<Response>) + Sync + Send>(
        &mut self,
        request: Request,
        callback: T,
    ) -> RequestCancellation {
        let (cancellation_signal_sender, cancellation_signal) = oneshot::channel();

        self.send_input_command(InputCommand::Request {
            cancellation_signal,
            request,
            callback: Box::new(callback),
        });

        RequestCancellation(cancellation_signal_sender)
    }

    fn send_input_command(&mut self, input_command: InputCommand) {
        let input_command_sender = self.input_command_sender.clone();
        self.number_of_pending_requests += 1;
        self.executor.spawn(futures::lazy(move || {
            input_command_sender
                .send(input_command)
                .map(|_| {})
                .map_err(|_| unreachable!())
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

        self.number_of_pending_requests -= 1;

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

    pub fn number_of_pending_requests(&self) -> usize {
        self.number_of_pending_requests
    }
}

mod tests {
    #[test]
    fn test_basic_request() {
        use super::*;
        use std::sync::{Arc, Mutex};

        let mut queue = Queue::new();

        use std::default::Default;

        let control_variable = Arc::new(Mutex::new(false));
        let control_variable_c = Arc::clone(&control_variable);
        let _handle = queue.send_request(
            RequestBuilder::default()
                .http_type(RequestType::Get)
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

    #[test]
    fn test_cancelling() {
        use super::*;
        use std::sync::{Arc, Mutex};

        let mut queue = Queue::new();

        use std::default::Default;

        let control_variable = Arc::new(Mutex::new(false));
        let control_variable_c = Arc::clone(&control_variable);
        let handle = queue.send_request(
            RequestBuilder::default()
                .http_type(RequestType::Get)
                .uri("https://docs.rs/".parse().unwrap())
                .build()
                .unwrap(),
            move |req| {
                *control_variable_c.lock().unwrap() = true;

                match req {
                    Ok(_) => {
                        unreachable!();
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::RequestCancelled => {}
                        _ => unreachable!(),
                    },
                };
            },
        );

        assert_eq!(*control_variable.lock().unwrap(), false);

        drop(handle);

        queue.execute_query_with_timeout(Duration::from_secs(5), Duration::from_millis(100));

        assert_eq!(*control_variable.lock().unwrap(), true);
    }

    #[test]
    fn test_timeout() {
        use super::*;
        use std::sync::{Arc, Mutex};

        let mut queue = Queue::new();

        use std::default::Default;

        let control_variable = Arc::new(Mutex::new(false));
        let control_variable_c = Arc::clone(&control_variable);
        let _handle = queue.send_request(
            RequestBuilder::default()
                .http_type(RequestType::Get)
                .options(
                    RequestOptionsBuilder::default()
                        .timeout(Some(Duration::new(0, 0)))
                        .build()
                        .unwrap(),
                )
                .uri("https://docs.rs/".parse().unwrap())
                .build()
                .unwrap(),
            move |req| {
                *control_variable_c.lock().unwrap() = true;

                match req {
                    Ok(_) => {
                        unreachable!();
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::RequestTimeout => {}
                        _ => unreachable!(),
                    },
                };
            },
        );

        assert_eq!(*control_variable.lock().unwrap(), false);

        queue.execute_query_with_timeout(Duration::from_secs(5), Duration::from_millis(100));

        assert_eq!(*control_variable.lock().unwrap(), true);
    }
}
