use std::fmt::Write;
use std::thread;
use std::time::Duration;

use eyre::Report;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::producer::{Producer, Record, RequiredAcks};

const KAFKA_HOSTS_ENV: &str = "KAFKA_HOSTS";
const KAFKA_TOPIC_NAME_ENV: &str = "my-topic";

fn main() {
    let hosts = std::env::var(KAFKA_HOSTS_ENV)
        .map_err(|e| Report::new(e))
        .and_then(|hosts| Ok(hosts.split(":")
            .map(|host| host.to_owned())
            .collect::<Vec<_>>()))
        .unwrap();


    create_and_start_producer(hosts.clone());
}

fn create_and_start_consumer(hosts: Vec<String>) {
    let mut consumer =
        Consumer::from_hosts(hosts)
            .with_topic_partitions(KAFKA_TOPIC_NAME_ENV.to_owned(), &[0, 1])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("my-group".to_owned())
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .create()
            .unwrap();
    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                println!("{:?}", m);
            }
            consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();
    }
}

fn create_and_start_producer(hosts: Vec<String>) {
    let mut producer =
        Producer::from_hosts(hosts)
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();


    thread::spawn(move || {
        let mut buf = String::with_capacity(2);
        for i in 0..10 {
            let _ = write!(&mut buf, "{}", i); // some computation of the message data to be sent
            producer.send(&Record::from_value(KAFKA_TOPIC_NAME_ENV, buf.as_bytes())).unwrap();
            buf.clear();
        }
    });
}

