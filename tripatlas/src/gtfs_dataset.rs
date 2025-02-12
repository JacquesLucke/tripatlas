use gtfs_io::Gtfs;

pub struct GtfsDataset {
    pub raw: Gtfs<'static>,
}
