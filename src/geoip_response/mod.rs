pub mod geoip_response {
    use serde::{Serialize, Deserialize};

    #[derive(Deserialize, Serialize)]
    pub struct GeoIpResponse {
        pub ip: GeoIpDataResponse,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub city: Option<GeoIpCityResponse>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub asn: Option<GeoIpAsnResponse>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GeoIpDataResponse {
        pub ip: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ptr: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GeoIpCityResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub state: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub country: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "countryIsoCode")]
        pub country_iso_code: Option<String>
    }

    #[derive(Serialize, Deserialize)]
    pub struct GeoIpAsnResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub number: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
    }
}