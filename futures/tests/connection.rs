extern crate lapin_futures as lapin;
extern crate futures;
extern crate tokio_core;

use futures::Stream;
use futures::future::Future;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

use lapin::types::FieldTable;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicConsumeOptions, BasicPublishOptions, BasicProperties,
                     QueueDeclareOptions, QueueDeleteOptions, QueuePurgeOptions};

#[test]
fn connection() {
    let mut core = Core::new().unwrap();

    let handle = core.handle();
    let addr = "127.0.0.1:5672".parse().unwrap();

    core.run(
        TcpStream::connect(&addr, &handle)
            .and_then(|stream| {
                lapin::client::Client::connect(stream, &ConnectionOptions::default())
            })
            .and_then(|client| {

                client
                    .create_channel()
                    .and_then(|channel| {
                        let id = channel.id;
                        println!("created channel with id: {}", id);

                        channel
                            .queue_declare(
                                "hello",
                                &QueueDeclareOptions::default(),
                                FieldTable::new(),
                            )
                            .and_then(move |_| {
                                println!("channel {} declared queue {}", id, "hello");

                                channel
                                    .queue_purge("hello", &QueuePurgeOptions::default())
                                    .and_then(move |_| {
                                        channel.basic_publish(
                                            "",
                                            "hello",
                                            b"hello from tokio",
                                            &BasicPublishOptions::default(),
                                            BasicProperties::default(),
                                        )
                                    })
                            })
                    })
                    .and_then(move |_| client.create_channel())
                    .and_then(|channel| {
                        let id = channel.id;
                        println!("created channel with id: {}", id);

                        let ch1 = channel.clone();
                        let ch2 = channel.clone();
                        channel
                            .queue_declare(
                                "hello",
                                &QueueDeclareOptions::default(),
                                FieldTable::new(),
                            )
                            .and_then(move |_| {
                                println!("channel {} declared queue {}", id, "hello");

                                channel.basic_consume(
                                    "hello",
                                    "my_consumer",
                                    &BasicConsumeOptions::default(),
                                )
                            })
                            .and_then(move |stream| {
                                println!("got consumer stream");

                                stream
                                    .into_future()
                                    .map_err(|(err, _)| err)
                                    .and_then(move |(message, _)| {
                                        let msg = message.unwrap();
                                        println!("got message: {:?}", msg);
                                        assert_eq!(msg.data, b"hello from tokio");
                                        ch1.basic_ack(msg.delivery_tag)
                                    })
                                    .and_then(move |_| {
                                        ch2.queue_delete("hello", &QueueDeleteOptions::default())
                                    })
                            })
                    })
            }),
    ).unwrap();
}
