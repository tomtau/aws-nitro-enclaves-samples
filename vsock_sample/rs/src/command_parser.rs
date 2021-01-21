use clap::ArgMatches;

#[derive(Debug, Clone)]
pub struct ServerArgs {
    pub port1: u32,
    pub port2: u32,
}

impl ServerArgs {
    pub fn new_with(args: &ArgMatches) -> Result<Self, String> {
        let (port1, port2) = parse_ports(args)?;
        Ok(ServerArgs { port1, port2 })
    }
}

#[derive(Debug, Clone)]
pub struct ClientArgs {
    pub port1: u32,
    pub port2: u32,
}

impl ClientArgs {
    pub fn new_with(args: &ArgMatches) -> Result<Self, String> {
        let (port1, port2) = parse_ports(args)?;
        Ok(ClientArgs { port1, port2 })
    }
}

fn parse_cid_client(args: &ArgMatches) -> Result<u32, String> {
    let cid = args.value_of("cid").ok_or("Could not find cid argument")?;
    cid.parse()
        .map_err(|_err| "cid is not a number".to_string())
}

fn parse_ports(args: &ArgMatches) -> Result<(u32, u32), String> {
    let port1 = args
        .value_of("port1")
        .ok_or("Could not find port argument")?;
    let port2 = args
        .value_of("port2")
        .ok_or("Could not find port argument")?;
    let p1 = port1
        .parse()
        .map_err(|_err| "port is not a number".to_string())?;
    let p2 = port2
        .parse()
        .map_err(|_err| "port is not a number".to_string())?;
    Ok((p1, p2))
}
