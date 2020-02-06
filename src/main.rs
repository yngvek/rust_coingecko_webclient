#![allow(unused_variables)]
use csv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use webclient::base_url;

fn main() -> Result<(), Box<dyn Error>> {
    let port = read_portfolio_from_file().unwrap();
    //println!("{:#?}", port?);

    let coin = get_simple_price_coin().unwrap();
    //println!("{:#?}", coin);

    let mut list_of_portfolio_coin: Vec<PortfolioCoin> = Vec::new();
    for p in port.iter() {
        for c in coin.iter() {
            if p.name == c.name {
                list_of_portfolio_coin.push(PortfolioCoin {
                    name: p.name.to_string(),
                    btc_value: c.btc_value,
                    usd_value: c.usd_value,
                    nok_value: c.nok_value,
                    amount: p.amount,
                    btc_value_total: c.btc_value * p.amount,
                    usd_value_total: (c.usd_value * p.amount) as i32,
                    nok_value_total: (c.nok_value * p.amount) as i32,
                    location: p.location.to_string(),
                });
            }
        }
    }

    println!("{:#?}", list_of_portfolio_coin);

    let sum: i32 = list_of_portfolio_coin
        .iter()
        .map(|c| c.nok_value_total as i32)
        .sum();
    println!("Sum i NOK er {}", sum);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct PortfolioCoin {
    name: String,
    btc_value: f64,
    usd_value: f64,
    nok_value: f64,
    amount: f64,
    btc_value_total: f64,
    usd_value_total: i32,
    nok_value_total: i32,
    location: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Coin {
    name: String,
    btc_value: f64,
    btc_24h_change: f64,
    usd_value: f64,
    usd_24h_change: f64,
    nok_value: f64,
    nok_24h_change: f64,
    updated: i32,
}
// impl PartialEq for Coin {
//     fn eq(&self, other: &Self) -> bool {
//         self.name == other.name
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
struct SimplePriceResponse(HashMap<String, HashMap<String, f64>>);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Portfolio {
    name: String,
    amount: f64,
    location: String,
}

fn read_portfolio_from_file() -> Result<Vec<Portfolio>, Box<dyn Error>> {
    let file = File::open("portfolio.csv")?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut portfolio_list: Vec<Portfolio> = Vec::new();

    for result in rdr.deserialize() {
        let record: Portfolio = result?;
        portfolio_list.push(record);
    }
    // println!("{:#?}", portfolio_list);
    Ok(portfolio_list)
}

fn ping() -> Result<(), Box<dyn Error>> {
    let base_url = base_url();
    let api_endpoint_ping = "/ping";
    let url = &(base_url + api_endpoint_ping);
    let resp = reqwest::get(url)?;

    if resp.status().is_success() {
        println!("success!");
    } else if resp.status().is_server_error() {
        println!("server error!");
    } else {
        println!("Something else happened. Status: {:?}", resp.status());
    }
    Ok(())
}

fn get_simple_price_coin() -> Result<Vec<Coin>, Box<dyn Error>> {
    let p = ping();
    let _p = match p {
        Ok(ok) => ok,
        Err(error) => panic!("There was a problem reaching coingecko: {:?}", error),
    };

    let base_url = base_url();
    let api_endpoint_simple_price: &str = "/simple/price?ids=";
    let api_endpoint_ending: &str =
        "&vs_currencies=usd,btc,nok&include_24hr_change=true&include_last_updated_at=true";

    let file = File::open("coin.csv")?;
    let file_contents = BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    for line in file_contents.lines() {
        lines.push(line.unwrap());
    }

    let mut query_string = String::from("");

    for line in lines.iter() {
        query_string.push_str(&line);
        query_string.push(',');
    }

    let url = &(base_url + api_endpoint_simple_price + &query_string + api_endpoint_ending);
    let mut resp = reqwest::get(url)?;
    let price: SimplePriceResponse = resp.json()?;

    //println!("Status: {}", resp.status());
    //println!("Headers:\n{:#?}", resp.headers());
    //println!("{:#?}", price);

    let savefile = OpenOptions::new()
        .append(true)
        .create(true)
        .open("coinprice.txt")?;
    let mut savefile_buf = BufWriter::new(savefile);

    let savefile_curr = OpenOptions::new()
        .write(true)
        .create(true)
        .open("coinpricecurr.txt")?;
    let mut savefile_cur_buf = BufWriter::new(savefile_curr);

    let mut list_of_coins: Vec<Coin> = Vec::new();
    for line in lines.iter() {
        println!(
            "{};{};{};{};{};{};{};{}",
            line,
            price.0[line]["btc"],
            price.0[line]["btc_24h_change"],
            price.0[line]["usd"],
            price.0[line]["usd_24h_change"],
            price.0[line]["nok"],
            price.0[line]["nok_24h_change"],
            price.0[line]["last_updated_at"]
        );
        writeln!(
            savefile_buf,
            "{};{};{};{};{};{};{};{}",
            line,
            price.0[line]["btc"],
            price.0[line]["btc_24h_change"],
            price.0[line]["usd"],
            price.0[line]["usd_24h_change"],
            price.0[line]["nok"],
            price.0[line]["nok_24h_change"],
            price.0[line]["last_updated_at"]
        )?;
        writeln!(
            savefile_cur_buf,
            "{};{};{};{};{};{};{};{}",
            line,
            price.0[line]["btc"],
            price.0[line]["btc_24h_change"],
            price.0[line]["usd"],
            price.0[line]["usd_24h_change"],
            price.0[line]["nok"],
            price.0[line]["nok_24h_change"],
            price.0[line]["last_updated_at"]
        )?;

        let coin = Coin {
            name: line.to_string(),
            btc_value: price.0[line]["btc"],
            btc_24h_change: price.0[line]["btc_24h_change"],
            usd_value: price.0[line]["usd"],
            usd_24h_change: price.0[line]["usd_24h_change"],
            nok_value: price.0[line]["nok"],
            nok_24h_change: price.0[line]["nok_24h_change"],
            updated: price.0[line]["last_updated_at"] as i32,
        };

        list_of_coins.push(coin);
        //println!("{:#?}", coin);
    }

    Ok(list_of_coins)
}
