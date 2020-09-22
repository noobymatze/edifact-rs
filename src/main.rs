/// Test
///
extern crate combine;

mod mig;

use std::include_str;
use std::path::PathBuf;
use structopt::StructOpt;
use std::fs::File;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "edifact",
    about = "An EDIFACT parser for the edi@energy subset"
)]
struct Opt {
    #[structopt(
        short = "f",
        long = "file",
        help = "EDIFACT file to be parsed",
        parse(from_os_str)
    )]
    file: PathBuf,
}

fn main() -> std::io::Result<()> {
    let interchange = include_str!("../APERAK.json");
    let opts = Opt::from_args();
    let desc: mig::description::Interchange =
        serde_json::from_str(interchange).expect("Works");
    // let interchange = mig::parse_file(opts.file).expect("Works2");
    // match_interchange(&desc, interchange);
    let mut file = File::open(opts.file)?;
    let result = mig::decode(vec![desc], &mut file);
    match result {
        Ok(interchange) => println!("{:?}", interchange),
        Err(errors) => println!("{}", errors)
    }

    Ok(())
}

// ""
//    let input = "UNA:+.? 'UNB+UNOC:3+9900467000000:500+9904590000002:500+200307:0705+C3AAAAAAAAHKLC'UNH+1+APERAK:D:07B:UN:2.1d'BGM+313+53ff5de4caab4ea18abafab5e6036991'DTM+137:202003070705:203'RFF+ACE:O1583553607732'DTM+171:202003070500:203'NAD+MS+9900467000000::293'NAD+MR+9904590000002::293'ERC+Z29'FTX+ABO+++LOC17251283352734'RFF+ACW:V1583553607732'RFF+AGO:9904590000002ORD1583553607706'FTX+AAO+++LOC Datenelement 3225 ungültiger Wert'FTX+Z02+++10'UNT+14+1'UNZ+1+C3AAAAAAAAHKLC'";