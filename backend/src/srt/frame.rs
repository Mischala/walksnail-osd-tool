use parse_display::FromStr;

#[derive(Debug, Clone)]
pub struct SrtFrame {
    pub start_time_secs: f32,
    pub end_time_secs: f32,
    pub data: Option<SrtFrameData>,
    pub debug_data: Option<SrtDebugFrameData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SrtFrameData {
    pub signal: Option<u8>,
    pub channel: Option<String>,
    pub flight_time: Option<u32>,
    pub sky_bat: Option<f32>,
    pub ground_bat: Option<f32>,
    pub latency: Option<u32>,
    pub bitrate_mbps: Option<f32>,
    pub distance: Option<u32>,
    pub hz: Option<u32>,
    pub sp: Option<u8>,
    pub gp: Option<u8>,
    pub air_temp: Option<u32>,
    pub gnd_temp: Option<u32>,
    pub ssnr: Option<f32>,
    pub gsnr: Option<f32>,
    pub stemp: Option<u32>,
    pub gtemp: Option<u32>,
    pub gerr: Option<u32>,
    pub serr: Option<u32>,
    pub sty_mode: Option<u32>,
    pub is_debug: bool,
}

impl std::str::FromStr for SrtFrameData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"(\w+):\s*([^:\s]+)").unwrap());

        let mut signal = None;
        let mut channel = None;
        let mut flight_time = None;
        let mut sky_bat = None;
        let mut ground_bat = None;
        let mut latency = None;
        let mut bitrate_mbps = None;
        let mut distance = None;
        let mut air_temp = None;
        let mut gnd_temp = None;
        let mut sty_mode = None;

        for cap in RE.captures_iter(s) {
            let key = &cap[1];
            let value = &cap[2];

            match key {
                "Signal" | "MCS" => signal = value.parse().ok(),
                "CH" => channel = Some(value.to_string()),
                "FlightTime" => flight_time = value.parse().ok(),
                "SBat" => sky_bat = value.trim_end_matches('V').parse().ok(),
                "GBat" => ground_bat = value.trim_end_matches('V').parse().ok(),
                "Delay" => latency = value.trim_end_matches("ms").parse().ok(),
                "Bitrate" => bitrate_mbps = value.trim_end_matches("Mbps").parse().ok(),
                "Distance" => distance = value.trim_end_matches('m').parse().ok(),
                "AirTemp" => air_temp = value.parse().ok(),
                "GndTemp" => gnd_temp = value.parse().ok(),
                "Stemp" => air_temp = value.parse().ok(),
                "Gtemp" => gnd_temp = value.parse().ok(),
                "STYMode" => sty_mode = value.parse().ok(),
                _ => {}
            }
        }

        Ok(Self {
            signal,
            channel,
            flight_time,
            sky_bat,
            ground_bat,
            latency,
            bitrate_mbps,
            distance,
            hz: None,
            sp: None,
            gp: None,
            air_temp,
            gnd_temp,
            ssnr: None,
            gsnr: None,
            stemp: None,
            gtemp: None,
            gerr: None,
            serr: None,
            sty_mode,
            is_debug: false,
        })
    }
}

#[derive(Debug, FromStr, Clone, PartialEq)]
#[display("Signal:{signal} CH:{channel} Hz:{hz} FlightTime:{flight_time} Sp={sp} Gp={gp} SBat:{sky_bat}V GBat:{ground_bat}V Delay:{latency}ms Bitrate:{bitrate_mbps}Mbps Distance:{distance}m")]
pub struct AscentSrtFrameData {
    pub signal: u8,
    pub channel: String,
    pub hz: u32,
    pub flight_time: u32,
    pub sp: u8,
    pub gp: u8,
    pub sky_bat: f32,
    pub ground_bat: f32,
    pub latency: u32,
    pub bitrate_mbps: f32,
    pub distance: u32,
}

impl From<AscentSrtFrameData> for SrtFrameData {
    fn from(ascent_data: AscentSrtFrameData) -> Self {
        Self {
            signal: Some(ascent_data.signal),
            channel: Some(ascent_data.channel),
            flight_time: Some(ascent_data.flight_time),
            sky_bat: Some(ascent_data.sky_bat),
            ground_bat: Some(ascent_data.ground_bat),
            latency: Some(ascent_data.latency),
            bitrate_mbps: Some(ascent_data.bitrate_mbps),
            distance: Some(ascent_data.distance),
            hz: Some(ascent_data.hz),
            sp: Some(ascent_data.sp),
            gp: Some(ascent_data.gp),
            air_temp: None,
            gnd_temp: None,
            ssnr: None,
            gsnr: None,
            stemp: None,
            gtemp: None,
            gerr: None,
            serr: None,
            sty_mode: None,
            is_debug: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AscentDebugSrtFrameData {
    pub hz: u32,
    pub signal: u8,
    pub ssnr: f32,
    pub gsnr: f32,
    pub stemp: u32,
    pub gtemp: u32,
    pub latency: u32,
    pub frame: u16,
    pub serr: u32,
    pub gerr: u32,
    pub sp: u8,
    pub gp: u8,
    pub bitrate_mbps: f32,
}

impl std::str::FromStr for AscentDebugSrtFrameData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(
                r"Hz:(?P<hz>\d+)\s+MCS:(?P<mcs>\d+).*SSNR:(?P<ssnr>[\d.]+)\s+GSNR:(?P<gsnr>[\d.]+)\s+Stemp:(?P<stemp>\d+)\s+Gtemp:(?P<gtemp>\d+)\s+Delay:(?P<delay>\d+)ms\s+Frame:(?P<frame>\d+)\s+Serr:(?P<serr>\d+)\s+Gerr:(?P<gerr>\d+).*Sp=(?P<sp>\d+)\s+Gp=(?P<gp>\d+)\s+(?P<bitrate>[\d.]+)",
            )
            .unwrap()
        });

        let caps = RE
            .captures(s)
            .ok_or_else(|| "Failed to match AscentDebugSrt pattern".to_string())?;

        Ok(Self {
            hz: caps["hz"].parse().map_err(|e| format!("hz: {e}"))?,
            signal: caps["mcs"].parse().map_err(|e| format!("mcs: {e}"))?,
            ssnr: caps["ssnr"].parse().map_err(|e| format!("ssnr: {e}"))?,
            gsnr: caps["gsnr"].parse().map_err(|e| format!("gsnr: {e}"))?,
            stemp: caps["stemp"].parse().map_err(|e| format!("stemp: {e}"))?,
            gtemp: caps["gtemp"].parse().map_err(|e| format!("gtemp: {e}"))?,
            latency: caps["delay"].parse().map_err(|e| format!("delay: {e}"))?,
            frame: caps["frame"].parse().map_err(|e| format!("frame: {e}"))?,
            serr: caps["serr"].parse().map_err(|e| format!("serr: {e}"))?,
            gerr: caps["gerr"].parse().map_err(|e| format!("gerr: {e}"))?,
            sp: caps["sp"].parse().map_err(|e| format!("sp: {e}"))?,
            gp: caps["gp"].parse().map_err(|e| format!("gp: {e}"))?,
            bitrate_mbps: caps["bitrate"].parse().map_err(|e| format!("bitrate: {e}"))?,
        })
    }
}

impl From<AscentDebugSrtFrameData> for SrtFrameData {
    fn from(d: AscentDebugSrtFrameData) -> Self {
        Self {
            signal: Some(d.signal),
            channel: None,
            flight_time: None,
            sky_bat: None,
            ground_bat: None,
            latency: Some(d.latency),
            bitrate_mbps: Some(d.bitrate_mbps),
            distance: None,
            hz: Some(d.hz),
            sp: Some(d.sp),
            gp: Some(d.gp),
            air_temp: Some(d.stemp),
            gnd_temp: Some(d.gtemp),
            ssnr: Some(d.ssnr),
            gsnr: Some(d.gsnr),
            stemp: Some(d.stemp),
            gtemp: Some(d.gtemp),
            gerr: Some(d.gerr),
            serr: Some(d.serr),
            sty_mode: None,
            is_debug: true,
        }
    }
}

#[derive(Debug, FromStr, Clone, PartialEq)]
#[display("CH:{channel} MCS:{signal} SP[ {sp1} {sp2}  {sp3} {sp4}] GP[ {gp1}  {gp2}  {gp3}  {gp4}] GTP:{gtp} GTP0:{gtp0} STP:{stp} STP0:{stp0} GSNR:{gsnr} SSNR:{ssnr} Gtemp:{gtemp} Stemp:{stemp} Delay:{latency}ms Frame:{frame}  Gerr:{gerr} SErr:{serr} {serr_ext}, [iso:{iso},mode={iso_mode}, exp:{iso_exp}] [gain:{gain} exp:{gain_exp}ms]")]
pub struct SrtDebugFrameData {
    pub signal: u8,
    pub channel: u8,
    //pub flight_time: u32,
    //pub sky_bat: f32,
    //pub ground_bat: f32,
    pub latency: u32,
    //pub bitrate_mbps: f32,
    //pub distance: u32,
    pub sp1: u16,
    pub sp2: u16,
    pub sp3: u16,
    pub sp4: u16,
    pub gp1: u16,
    pub gp2: u16,
    pub gp3: u16,
    pub gp4: u16,
    pub gtp: u16,
    pub gtp0: u16,
    pub stp: u16,
    pub stp0: u16,
    pub gsnr: f32,
    pub ssnr: f32,
    pub gtemp: f32,
    pub stemp: f32,
    pub frame: u16,
    pub gerr: u16,
    pub serr: u16,
    pub serr_ext: u16,
    pub iso: u32,
    pub iso_mode: String,
    pub iso_exp: u32,
    pub gain: f32,
    pub gain_exp: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pre_v31_36_8_srt_frame_data() {
        let line = "Signal:4 CH:8 FlightTime:0 SBat:4.7V GBat:7.2V Delay:32ms Bitrate:25Mbps Distance:7m";
        let parsed = line.parse::<SrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtFrameData {
                signal: Some(4),
                channel: Some("8".to_string()),
                flight_time: Some(0),
                sky_bat: Some(4.7),
                ground_bat: Some(7.2),
                latency: Some(32),
                bitrate_mbps: Some(25.0),
                distance: Some(7),
                hz: None,
                sp: None,
                gp: None,
                air_temp: None,
                gnd_temp: None,
                ssnr: None,
                gsnr: None,
                stemp: None,
                gtemp: None,
                gerr: None,
                serr: None,
                sty_mode: None,
                is_debug: false,
            }
        );
    }

    #[test]
    fn parse_v32_37_10_srt_frame_data() {
        let line = "Signal:4 CH:7 FlightTime:0 SBat:16.7V GBat:12.5V Delay:25ms Bitrate:25.0Mbps Distance:1m";
        let parsed = line.parse::<SrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtFrameData {
                signal: Some(4),
                channel: Some("7".to_string()),
                flight_time: Some(0),
                sky_bat: Some(16.7),
                ground_bat: Some(12.5),
                latency: Some(25),
                bitrate_mbps: Some(25.0),
                distance: Some(1),
                hz: None,
                sp: None,
                gp: None,
                air_temp: None,
                gnd_temp: None,
                ssnr: None,
                gsnr: None,
                stemp: None,
                gtemp: None,
                gerr: None,
                serr: None,
                sty_mode: None,
                is_debug: false,
            }
        );
    }

    #[test]
    fn parse_v37_42_3_debug_src_frame_data() {
        let line = "CH:1 MCS:4 SP[ 45 152  47 149] GP[ 49  48  45  47] GTP:27 GTP0:00 STP:24 STP0:00 GSNR:25.9 SSNR:17.8 Gtemp:50 Stemp:82 Delay:31ms Frame:60  Gerr:0 SErr:0 42, [iso:0,mode=max, exp:0] [gain:0.00 exp:0.000ms]";
        let parsed = line.parse::<SrtDebugFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtDebugFrameData {
                signal: 4,
                channel: 1,
                //flight_time: 0,
                //sky_bat: 0,
                //ground_bat: 0,
                latency: 31,
                //bitrate_mbps: 0,
                //distance: 0,
                sp1: 45,
                sp2: 152,
                sp3: 47,
                sp4: 149,
                gp1: 49,
                gp2: 48,
                gp3: 45,
                gp4: 47,
                gtp: 27,
                gtp0: 0,
                stp: 24,
                stp0: 0,
                gsnr: 25.9,
                ssnr: 17.8,
                gtemp: 50.0,
                stemp: 82.0,
                frame: 60,
                gerr: 0,
                serr: 0,
                serr_ext: 42,
                iso: 0,
                iso_mode: "max".to_string(),
                iso_exp: 0,
                gain: 0.0,
                gain_exp: 0.0
            }
        );
    }

    #[test]
    fn parse_ascent_srt_frame_data() {
        let line = "Signal:4 CH:AUTO Hz:5805000 FlightTime:0 Sp=19 Gp=17 SBat:5.0V GBat:11.6V Delay:37ms Bitrate:25.0Mbps Distance:0m";
        let parsed = line.parse::<AscentSrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse Ascent SRT frame data"),
            AscentSrtFrameData {
                signal: 4,
                channel: "AUTO".to_string(),
                hz: 5_805_000,
                flight_time: 0,
                sp: 19,
                gp: 17,
                sky_bat: 5.0,
                ground_bat: 11.6,
                latency: 37,
                bitrate_mbps: 25.0,
                distance: 0
            }
        );
    }

    #[test]
    fn parse_artlynk_srt_frame_data() {
        let line = "Signal:4 CH: 3 FlightTime:   0 SBat:7.11 GBat:7.54 Bitrate: 4Mbps Distance:     0m STYMode:1 AirTemp: 49 GndTemp: 34";
        let parsed = line.parse::<SrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse Artlynk SRT frame data"),
            SrtFrameData {
                signal: Some(4),
                channel: Some("3".to_string()),
                flight_time: Some(0),
                sky_bat: Some(7.11),
                ground_bat: Some(7.54),
                latency: None,
                bitrate_mbps: Some(4.0),
                distance: Some(0),
                hz: None,
                sp: None,
                gp: None,
                air_temp: Some(49),
                gnd_temp: Some(34),
                ssnr: None,
                gsnr: None,
                stemp: None,
                gtemp: None,
                gerr: None,
                serr: None,
                sty_mode: Some(1),
                is_debug: false,
            }
        );
    }

    #[test]
    fn test_ascent_parsing_priority() {
        let line = "Signal:4 CH:AUTO Hz:5805000 FlightTime:0 Sp=19 Gp=17 SBat:5.0V GBat:11.6V Delay:37ms Bitrate:25.0Mbps Distance:0m";
        let mut data = line.parse::<AscentSrtFrameData>().ok().map(SrtFrameData::from);
        if data.is_none() {
            data = line.parse::<SrtFrameData>().ok();
        }
        let data = data.expect("Should parse as AscentSrtFrameData");
        assert_eq!(data.hz, Some(5_805_000));
        assert_eq!(data.sp, Some(19));
        assert_eq!(data.gp, Some(17));
    }
}
