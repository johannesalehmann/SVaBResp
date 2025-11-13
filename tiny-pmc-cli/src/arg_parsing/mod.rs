use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long)]
    pub model: String,
    #[arg(short, long)]
    pub property: String,
    #[arg(short, long, default_value_t = String::new())]
    pub constants: String,
}
