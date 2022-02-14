//! In-memory key/value store application for Tendermint.

use structopt::StructOpt;
use tendermint_abci::{KeyValueStoreApp, ServerBuilder};
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Bind the TCP server to this host.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Bind the TCP server to this port.
    #[structopt(short, long, default_value = "26658")]
    port: u16,

    /// The default server read buffer size, in bytes, for each incoming client
    /// connection.
    #[structopt(short, long, default_value = "1048576")]
    read_buf_size: usize,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress all output logging (overrides --verbose).
    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
    // let opt: Opt = Opt::from_args();
    // let log_level = if opt.quiet {
    //     LevelFilter::OFF
    // } else if opt.verbose {
    //     LevelFilter::DEBUG
    // } else {
    //     LevelFilter::INFO
    // };
    // tracing_subscriber::fmt().with_max_level(log_level).init();
    //
    // let (app, driver) = KeyValueStoreApp::new();
    // let server = ServerBuilder::new(opt.read_buf_sizeSubscription)
    //     .bind(format!("{}:{}", opt.host, opt.port), app)
    //     .unwrap();
    // std::thread::spawn(move || driver.run());
    // server.listen().unwrap();
    use futures::StreamExt;
    use tendermint_rpc::Subscription;

    /// Prints `count` events from the given subscription.
    async fn print_events(subs: &mut Subscription, count: usize) {
        let mut counter = 0_usize;
        while let Some(res) = subs.next().await {
            // Technically, a subscription produces `Result<Event, Error>`
            // instances. Errors can be produced by the remote endpoint at any
            // time and need to be handled here.
            let ev = res.unwrap();
            println!("Got incoming event: {:?}", ev);
            counter += 1;
            if counter >= count {
                break;
            }
        }
    }
}
