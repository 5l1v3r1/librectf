use web;
use Config;

#[derive(Debug, StructOpt)]
pub struct Web {
    #[structopt(flatten)]
    config: Config,
}

impl Web {
    pub fn run(&self) {
        let app = web::app(&self.config);
        app.launch();
    }
}