# GoldSrc Rest In Pawn (gRIP) [![Build Status](https://travis-ci.com/In-line/grip.svg?branch=master)](https://travis-ci.com/In-line/grip)

GRIP started as an effort to make an asynchronous REST API for Pawn.  

Months ago I was invited to participate in the GameX project initiative. GameX was striving to create a modern administrating system for HLDS. To do this many problems queued up, waiting to be solved. One of the major challenges was the interaction with the database. 

A long time ago before GRIP and GameX, web-server and HLDS modified database direcly. It was a crude and a brutal approach, which is one of the sources of todays problems. 
GRIP was created to fill this gap, by providing asynchronous REST Client for Pawn. Database access and modification, with the server-specific code are deligated to the server and encapsulated in REST API, which is a sophysticated and modern approach.

Before we proceed, I should mention that HLDS and Pawn operate on the main thread. Generally code isn't concurrent. 

So to sum up there are 3 key requirements for the GRIP. 
1) Low-latency
2) Asynchronous API
3) Support for the modern web standards.

Solutions implemented before, usually achieved only second requirement or just basically nothing. They used raw sockets and blocked the main thread for the IO. 
To solve blocking problem requests were dispatched to another thread (See for SQL_ThreadQuery for example). This approach has several problems. One of them is the difficulty on concurrent and safe programming in C++. To speed up things, thread pool should be written. To write a fast thread pool you need to be a highly professional programmer... This is just a mess! 

Why don't use modern programming language which is as fast as C++, but is safe, has greater libraries and tooling support? Yeah, basically no reasons to not to.  

Rust has many libraries dedicated to high-performance HTTP(s) requests, implemented (at the lowest level) using non-blocking sockets. It is extremely easy to write something in Rust, which is otherwise nearly impossible in C++. 

Thus, I, with the team of dedicated supporters give a birth to a GRIP. It supports modern web standards, will eventually support JSON and many other things..
