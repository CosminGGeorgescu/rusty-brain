pub mod brainvision_core {
    // * https://www.brainproducts.com/download/specification-of-brainvision-core-data-format-1-0/

    use core::str;
    use std::{fmt::Display, fs, io::Read, path::Path, str::Split};

    use ndarray::{Array2, Shape};

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

        pub fn get_data_file(&self) -> &str {
            &self.data_file
        }

        pub fn get_marker_file(&self) -> &str {
            &self.marker_file
        }

        pub fn get_binary_format(&self) -> BinaryFormat {
            self.binary_format
        }

        pub fn get_num_channels(&self) -> u32 {
            self.num_channels
        }
    }

    #[derive(Clone, Copy)]
    pub enum BinaryFormat {
        IeeeFloat32 = 4,
        Int16 = 2,
    }

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
        date: Option<Date>,
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

    pub struct Date {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
    }

    impl From<&str> for Date {
        fn from(value: &str) -> Self {
            let year = value[0..4].parse::<u16>().unwrap();
            let month = value[4..6].parse::<u8>().unwrap();
            let day = value[6..8].parse::<u8>().unwrap();
            let hour = value[8..10].parse::<u8>().unwrap();
            let minute = value[10..12].parse::<u8>().unwrap();
            let second = value[12..14].parse::<u8>().unwrap();
            let microsecond = value[14..20].parse::<u32>().unwrap();

            Self {
                year,
                month,
                day,
                hour,
                minute,
                second,
                microsecond,
            }
        }
    }

    impl Display for Date {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let month = match self.month {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => return Err(std::fmt::Error),
            };

            write!(
                f,
                "{} {} {}, {}:{}:{}.{}",
                self.day, month, self.year, self.hour, self.minute, self.second, self.microsecond
            )
        }
    }

    pub struct RawData {
        header: Header,
        marker: Marker,
        data: Data,
    }

    impl RawData {
        pub fn load<P>(
            root: P,
            subject: &str,
            session: Option<&str>,
            task: &str,
            acquisition: Option<&str>,
            run: Option<&str>,
        ) -> RawData
        where
            P: AsRef<Path>,
        {
            let mut base_path = root.as_ref().join(format!("sub-{}", subject));
            if let Some(session) = session {
                base_path.push(format!("ses-{}", session));
            }
            base_path.push("eeg");

            let mut files_path = format!("sub-{}", subject);
            if let Some(session) = session {
                files_path += &format!("_ses-{}", session);
            }
            files_path.push_str(&format!("_task-{}", task));
            if let Some(acquisition) = acquisition {
                files_path += &format!("_acq-{}", acquisition);
            }
            if let Some(run) = run {
                files_path += &format!("_run-{}", run);
            }

            let (header, marker) = (
                Header::load(base_path.join(format!("{}_eeg.vhdr", files_path))),
                Marker::load(base_path.join(format!("{}_eeg.vmrk", files_path))),
            );

            let rawdata = fs::read(base_path.join(header.get_data_file())).unwrap();
            let (num_channels, data_bytes_size) = (
                header.get_num_channels() as usize,
                header.get_binary_format(),
            );
            let chunks = rawdata.chunks_exact(data_bytes_size as usize);
            // ! Actual number of data points is `effective_data_points` + 1
            let effective_data_points = chunks.len() / num_channels;
            println!("({}, {})", num_channels, effective_data_points);
            let data = match data_bytes_size {
                BinaryFormat::IeeeFloat32 => Data::IeeeFloat32(
                    Array2::from_shape_vec(
                        (num_channels, effective_data_points),
                        chunks
                            .map(|x| {
                                let mut rep = [0u8; 4];
                                rep.copy_from_slice(x);
                                f32::from_le_bytes(rep)
                            })
                            .collect::<Vec<f32>>(),
                    )
                    .unwrap(),
                ),
                BinaryFormat::Int16 => Data::Int16(
                    Array2::from_shape_vec(
                        (num_channels, effective_data_points),
                        chunks
                            .map(|x| {
                                let mut rep = [0u8; 2];
                                rep.copy_from_slice(x);
                                u16::from_le_bytes(rep)
                            })
                            .collect::<Vec<u16>>(),
                    )
                    .unwrap(),
                ),
            };

            RawData {
                header,
                marker,
                data,
            }
        }
    }

    enum Data {
        IeeeFloat32(Array2<f32>),
        Int16(Array2<u16>),
    }
}
