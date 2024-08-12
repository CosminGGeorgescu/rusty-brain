pub mod brainvision_core {
    // * https://www.brainproducts.com/download/specification-of-brainvision-core-data-format-1-0/

    use std::{io::Read, path::Path, str::Split};

    pub struct Header {
        data_file: String,
        marker_file: String,
        num_channels: u32,
        sampling_interval: f64,
        averaged: bool,
        averaged_segms: u32,
        num_data_points: u32,
        segmentation_type: String,
        binary_format: BinaryFormat,
        channels: Vec<ChannelInfo>,
        channel_coords: Option<Vec<Coordinates>>,
        comment: Option<String>,
    }

    impl Header {
        pub fn load<P>(path: P) -> Header
        where
            P: AsRef<Path>,
        {
            let mut buf = String::new();
            let _ = std::fs::File::open(path.as_ref())
                .unwrap()
                .read_to_string(&mut buf);
            let comment = buf
                .match_indices("[Comment]")
                .next()
                .map(|(idx, _)| buf[idx + "[Comment]".len()..buf.len()].to_string());
            buf = buf.lines().skip(1).collect::<Vec<&str>>().join("\n");

            let file = ini::Ini::load_from_str(&buf).unwrap();

            let common_infos = file.section(Some("Common Infos")).unwrap();
            let binary_infos = file.section(Some("Binary Infos")).unwrap();
            let channel_infos = file.section(Some("Channel Infos")).unwrap();
            let coordinates = file.section(Some("Coordinates"));

            let data_file = common_infos.get("DataFile").unwrap().into();
            let marker_file = common_infos.get("MarkerFile").unwrap().into();
            let num_channels = common_infos
                .get("NumberOfChannels")
                .map(|s| s.parse::<u32>().unwrap())
                .unwrap();
            let sampling_interval = common_infos
                .get("SamplingInterval")
                .map(|s| s.parse::<f64>().unwrap())
                .unwrap();
            let averaged = common_infos.get("Averaged").map_or_else(
                || false,
                |s| match s {
                    "YES" => true,
                    "NO" | _ => false,
                },
            );
            let averaged_segms = match averaged {
                false => 0u32,
                true => common_infos
                    .get("AveragedSegments")
                    .map(|s| s.parse::<u32>().unwrap())
                    .unwrap(),
            };
            let segmentation_type = match averaged {
                false => "NOTSEGMENTED",
                true => common_infos.get("SegmentationType").unwrap(),
            }
            .into();
            let num_data_points = if averaged && segmentation_type == "MARKERBASED" {
                common_infos
                    .get("SegmentDataPoints")
                    .map(|s| s.parse::<u32>().unwrap())
                    .unwrap()
            } else {
                0
            };

            let binary_format = match binary_infos.get("BinaryFormat").unwrap() {
                "IEEE_FLOAT_32" => BinaryFormat::IeeeFloat32,
                "INT_16" => BinaryFormat::Int16,
                _ => panic!("Invalid binary format !"),
            };

            let channels = channel_infos
                .iter()
                .map(|(_, v)| ChannelInfo::from(v.split(',')))
                .collect::<Vec<ChannelInfo>>();

            let channel_coords = coordinates.map(|coords| {
                coords
                    .iter()
                    .map(|(_, v)| Coordinates::from(v.split(',')))
                    .collect::<Vec<Coordinates>>()
            });

            Header {
                data_file,
                marker_file,
                num_channels,
                sampling_interval,
                averaged,
                averaged_segms,
                num_data_points,
                segmentation_type,
                binary_format,
                channels,
                channel_coords,
                comment,
            }
        }
    }

    enum BinaryFormat {
        IeeeFloat32 = 0,
        Int16 = 1,
    }

    #[derive(Debug)]
    struct ChannelInfo {
        name: String,
        ref_name: String,
        resolution: f64,
        unit: String,
    }

    impl From<Split<'_, char>> for ChannelInfo {
        fn from(mut value: Split<char>) -> Self {
            let name = value.next().unwrap().into();
            let ref_name = value
                .next()
                .map(|s| if s.is_empty() { "Cz" } else { s })
                .unwrap()
                .into();
            let resolution = value
                .next()
                .map(|s| {
                    if s.is_empty() {
                        1.0f64
                    } else {
                        s.parse::<f64>().unwrap()
                    }
                })
                .unwrap();
            let unit = value
                .next()
                .map(|s| if s.is_empty() { "μV" } else { s })
                .unwrap()
                .into();

            Self {
                name,
                ref_name,
                resolution,
                unit,
            }
        }
    }

    struct Coordinates {
        radius: f64,
        theta: f64,
        phi: f64,
    }

    impl From<Split<'_, char>> for Coordinates {
        fn from(mut value: Split<'_, char>) -> Self {
            Coordinates {
                radius: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
                theta: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
                phi: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
            }
        }
    }

    pub struct Marker {
        data_file: String,
        markers: Vec<MarkerInfo>,
    }

    impl Marker {
        pub fn load<P>(path: P) -> Marker
        where
            P: AsRef<Path>,
        {
            let mut buf = String::new();
            let _ = std::fs::File::open(path.as_ref())
                .unwrap()
                .read_to_string(&mut buf);
            buf = buf.lines().skip(1).collect::<Vec<&str>>().join("\n");

            let file = ini::Ini::load_from_str(&buf).unwrap();

            let common_infos = file.section(Some("Common Infos")).unwrap();
            let marker_infos = file.section(Some("Marker Infos")).unwrap();

            let data_file = common_infos.get("DataFile").unwrap().into();

            let markers = marker_infos
                .iter()
                .map(|(_, v)| MarkerInfo::from(v.split(',')))
                .collect::<Vec<MarkerInfo>>();

            Self { data_file, markers }
        }
    }

    struct MarkerInfo {
        r#type: String,
        description: String,
        position: u32,
        points: u32,
        nr: i32,
        date: Option<String>,
    }

    impl From<Split<'_, char>> for MarkerInfo {
        fn from(mut value: Split<'_, char>) -> Self {
            let r#type = value.next().unwrap().into();
            let description = value.next().unwrap().into();
            let position = value.next().map(|s| s.parse::<u32>().unwrap()).unwrap();
            let points = value.next().map(|s| s.parse::<u32>().unwrap()).unwrap();
            let nr = value.next().map(|s| s.parse::<i32>().unwrap()).unwrap();
            let date = value.next().map(|s| s.into());

            Self {
                r#type,
                description,
                position,
                points,
                nr,
                date,
            }
        }
    }
}
