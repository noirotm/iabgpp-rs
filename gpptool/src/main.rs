use clap::{Parser, Subcommand};
use colored_json::{Color, ColorMode, Output, Styler, ToColoredJson};
use iab_gpp::sections::SectionId;
use iab_gpp::v1::GPPString;
use num_traits::cast::FromPrimitive;
use num_traits::ToPrimitive;
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a GPP string and display it in the console
    Parse {
        /// GPP string to parse
        gpp_string: String,
        /// Section ID to parse
        #[arg(short, long)]
        section_id: Option<u32>,
    },
    /// List all sections
    List {
        /// GPP string to parse
        gpp_string: String,
    },
}

fn main() {
    let args = Cli::parse();

    let e = match args.cmd {
        Commands::Parse {
            gpp_string,
            section_id: None,
        } => parse_gpp_string(&gpp_string),
        Commands::Parse {
            gpp_string,
            section_id: Some(id),
        } => parse_gpp_string_section(&gpp_string, id),
        Commands::List { gpp_string } => list_sections(&gpp_string),
    };

    if let Err(e) = e {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn parse_gpp_string(s: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gpp_str = GPPString::from_str(s)?;

    let sections = gpp_str
        .decode_all_sections()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    println!(
        "{}",
        serde_json::to_string_pretty(&sections)?
            .to_colored_json_with_styler(ColorMode::Auto(Output::StdOut), json_color_styler())?
    );

    Ok(())
}

fn parse_gpp_string_section(s: &str, id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let gpp_str = GPPString::from_str(s)?;

    let section = gpp_str.decode_section(SectionId::from_u32(id).ok_or("Invalid ID")?)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&section)?
            .to_colored_json_with_styler(ColorMode::Auto(Output::StdOut), json_color_styler())?
    );

    Ok(())
}

fn list_sections(s: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gpp_str = GPPString::from_str(s)?;

    for s in gpp_str.section_ids() {
        println!("{}\t{:?}", s.to_u32().unwrap_or_default(), s);
    }

    Ok(())
}

fn json_color_styler() -> Styler {
    Styler {
        key: Color::Green.foreground(),
        string_value: Color::Blue.bold(),
        integer_value: Color::Magenta.bold(),
        float_value: Color::Magenta.italic(),
        object_brackets: Color::Yellow.bold(),
        array_brackets: Color::Cyan.bold(),
        ..Default::default()
    }
}
