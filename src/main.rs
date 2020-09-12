#![deny(warnings)]

#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use std::env;
use warp::Filter;
use std::net::IpAddr;
use maxminddb::{geoip2, Reader, MaxMindDBError};
use dns_lookup::{lookup_host, lookup_addr};

mod geoip_response;

use crate::geoip_response::geoip_response::{GeoIpResponse, GeoIpDataResponse, GeoIpCityResponse, GeoIpAsnResponse};
use std::time::Instant;

lazy_static! {
    static ref READER_ASN: Reader<Vec<u8>> = maxminddb::Reader::open_readfile("GeoLite2-ASN.mmdb").unwrap();
    static ref READER_CITY: Reader<Vec<u8>> = maxminddb::Reader::open_readfile("GeoLite2-City.mmdb").unwrap();
}

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();

    let geoip_route = warp::post()
        .and(warp::path!("api" / "v1" / "geoip"))
        .and(warp::body::json())
        .map(|ips: Vec<String>| {
            let now = Instant::now();

            info!("Received an GeoIP request for IPs: {:?}", ips);

            let reply = warp::reply::json(&ip_to_geoip(ips));

            info!("Finished GeoIP requests. Elapsed time: {:?}", now.elapsed());

            return reply;
        });

    warp::serve(geoip_route).run(([127, 0, 0, 1], 7881)).await;
}

fn ip_to_geoip(ips: Vec<String>) -> Vec<GeoIpResponse> {
    let mut array_geoip: Vec<GeoIpResponse> = vec![];

    for ip_addr in ips.iter() {
        let now = Instant::now();

        info!("Geolocating IP {}", ip_addr);

        let ip: IpAddr = lookup_host(ip_addr).unwrap().first().unwrap().to_owned();

        if ip.to_string().ne(ip_addr) {
            info!("Resolved DNS {} to IP {}", ip_addr, ip.to_string())
        }
        let ptr_dns = lookup_addr(&ip).unwrap().to_string();
        let ptr = if ptr_dns.eq(&ip.to_string()) { None } else { Option::from(ptr_dns) };
        let asn_option: Result<geoip2::Asn, MaxMindDBError> = READER_ASN.lookup(ip);
        let city_option: Result<geoip2::City, MaxMindDBError> = READER_CITY.lookup(ip);

        let mut city_name: Option<String> = None;
        let mut state_name: Option<String> = None;
        let mut country_name: Option<String> = None;
        let mut country_iso_code: Option<String> = None;

        let mut asn_number: Option<String> = None;
        let mut asn_name: Option<String> = None;

        match city_option {
            Ok(city) => {
                if let Some(i) = &city.city {
                    city_name = Option::from(i.names.as_ref().unwrap().get("en").unwrap().to_owned())
                } else {
                    error!("No City found for IP: {}", ip);
                }

                if let Some(i) = &city.subdivisions {
                    state_name = Option::from(i.first().unwrap().names.as_ref().unwrap().get("en").unwrap().to_owned())
                } else {
                    error!("No State found for IP: {}", ip);
                }

                if let Some(i) = &city.country {
                    country_name = Option::from(i.names.as_ref().unwrap().get("en").unwrap().to_owned())
                } else {
                    error!("No Country found for IP: {}", ip);
                }

                if let Some(i) = &city.country {
                    country_iso_code = Option::from(i.iso_code.as_ref().unwrap().to_owned())
                } else {
                    error!("No Country ISO code found for IP: {}", ip);
                }
            }
            Err(err) => error!("An error happened while searching City for IP: {}, {}", ip, err),
        }

        match asn_option {
            Ok(asn) => {
                asn_number = Option::from(format!("AS{}", asn.autonomous_system_number.unwrap().to_string()));
                asn_name = asn.autonomous_system_organization;
            }
            Err(err) => error!("An error happened while searching ASN for IP: {}, {}", ip, err),
        }

        let response = GeoIpResponse {
            ip: GeoIpDataResponse {
                ip: (ip.to_string()).parse().unwrap(),
                ptr,
            },
            city: if let Some(_v) = &city_name {
                Option::from(GeoIpCityResponse {
                    name: city_name,
                    state: state_name,
                    country: country_name,
                    country_iso_code,
                })
            } else { None },
            asn: if let Some(_v) = &asn_number {
                Option::from(GeoIpAsnResponse {
                    number: asn_number,
                    name: asn_name,
                })
            } else { None },
        };

        array_geoip.push(response);

        info!("Done geolocalization of IP: {}. Elapsed time: {:?}", ip, now.elapsed());
    }

    return array_geoip;
}