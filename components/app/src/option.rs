use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "d", long)]
    debug: bool,
    #[structopt(short = "p", default_value = "3000")]
    serve_port: u16,
    #[structopt(short = "gp", default_value = "3001")]
    gossip_port: u16,
    #[structopt(short = "ll", default_value = "info")]
    log_level: String,
}
