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

use futures::prelude::*;
use futures::sync::mpsc as fmpsc;
use hyper::rt::*;
use std::sync::mpsc as smpsc;

enum Command {
    Request(String),
    Quit,
}

pub struct Queue {
    working_thread: thread::JoinHandle<()>,
    executor: tokio::runtime::TaskExecutor,
    command_sender: fmpsc::UnboundedSender<Command>, // TODO:
    response_receiver: crossbeam_channel::Receiver<String>, // TODO
}

impl Queue {
    pub fn new() -> Self {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let executor = runtime.executor();
        let (command_sender, command_receiver) = fmpsc::unbounded();
        let (response_sender, response_receiver) = crossbeam_channel::unbounded();

        let client = {
            let https = hyper_tls::HttpsConnector::new(4); // TODO: Number of DNS threads?
            crate::client::Client::new(
                hyper::Client::builder()
                    .executor(executor.clone())
                    .build::<_, hyper::Body>(https.unwrap()),
            )
        };

        let working_thread = {
            let executor = executor.clone();
            thread::spawn(move || {
                runtime
                    .block_on(lazy(move || {
                        command_receiver
                            .take_while(|cmd| {
                                Ok(match cmd {
                                    Command::Quit => false,
                                    _ => true,
                                })
                            }).for_each(move |cmd| {
                                match cmd {
                                    Command::Quit => unreachable!(),
                                    Command::Request(str) => {
                                        // TODO:
                                        use std::str::FromStr;
                                        executor.spawn(
                                            client
                                                .get(hyper::Uri::from_str(&str).unwrap())
                                                .map(|_| {})
                                                .map_err(|_| {}),
                                        )
                                    }
                                }

                                Ok(())
                            })
                    })).unwrap();
            })
        };

        Queue {
            working_thread,
            executor,
            command_sender,
            response_receiver,
        }
    }

    pub fn execute_queue(&mut self, limit: usize) -> usize {
        let mut counter = 0;

        loop {
            match self.response_receiver.try_recv() {
                Ok(response) => {
                    // TODO:
                    counter += 1;
                }
                Err(_) => break,
            }
        }
        counter
    }
}
//        NetworkingQueue {
//            working_thread:
