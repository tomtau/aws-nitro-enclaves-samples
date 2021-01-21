use log::error;

pub trait ExitGracefully<T, E> {
    fn ok_or_exit(self, message: &str) -> T;
}

impl<T, E: std::fmt::Debug> ExitGracefully<T, E> for Result<T, E> {
    fn ok_or_exit(self, message: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                error!("{:?}: {}", err, message);
                std::process::exit(1);
            }
        }
    }
}

#[macro_export]
macro_rules! create_app {
    () => {
        App::new("Vsock Sample")
            .about("Hello world example for vsock server and client communication.")
            .setting(AppSettings::ArgRequiredElseHelp)
            .version(env!("CARGO_PKG_VERSION"))
            .subcommand(
                SubCommand::with_name("server")
                    .about("Listen on given ports.")
                    .arg(
                        Arg::with_name("port1")
                            .long("port1")
                            .help("port")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name("port2")
                            .long("port2")
                            .help("port")
                            .takes_value(true)
                            .required(true),
                    ),,
            )
            .subcommand(
                SubCommand::with_name("client")
                    .about("Connect to given ports.")
                    .arg(
                        Arg::with_name("port1")
                            .long("port1")
                            .help("port")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name("port2")
                            .long("port2")
                            .help("port")
                            .takes_value(true)
                            .required(true),
                    ),
            )
    };
}
