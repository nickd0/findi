// Common mDNS-discovered services

use lazy_static::lazy_static;
use super::service::Service;

// TODO: make this hash ds into a struct `ServiceGroup` that has a name and vec of services
lazy_static! {
	pub static ref DEFAULT_SERVICES: Vec<(&'static str, Vec<Service>)> = {
		vec![
			("All services", vec![]),
			("Spotify Connect", vec![Service::new("Spotify Connect", "_spotify-connect")]),
			("Apple", vec![Service::new("Airport", "_airport")]),
			(
				"Printing",
				vec![
					Service::new("Universal", "_universal._sub._ipps"),
					Service::new("Fax IPP", "_fax-ipp"),
				]
			),
			(
				"Home devices",
				vec![
					Service::new("Fax IPP", "_fax-ipp"),
					Service::new("Google Cast (Chromecast)", "_googlecast._tcp.local"),
					Service::new("Google Zone (Chromecast)", "_googlezone._tcp.local"),
					Service::new("Apple HomeKit – HomeKit Accessory Protocol", "_hap._tcp.local"),
					Service::new("Apple HomeKit", "_homekit._tcp.local"),
					Service::new("iTunes Home Sharing", "_home-sharing._tcp.local"),
					Service::new("Apple TV Home Sharing", "_appletv-v2._tc"),
					Service::new("Amazon Devices", "_amzn-wplay._tcp.local"),
				]
			),
		]
	};
}
