use crate::phys_const::phys_const;

pub struct LlePoint {
    lat: f64,
    lon: f64,
    elevation: f64,
}

pub struct EcefPoint {
    x: f64,
    y: f64,
    z: f64,
}

pub struct EnuPoint {
    e: f64,
    n: f64,
    u: f64,
}
impl EcefPoint {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        EcefPoint { x, y, z }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn to_lle(&self) -> LlePoint {
        let a = phys_const::EARTH_SEMI_MAJOR_AXIS;
        let b = phys_const::EARTH_SEMI_MINOR_AXIS;
        let e2 = (a * a - b * b) / (a * a);
        let p = (self.x * self.x + self.y * self.y).sqrt();
        let theta = (self.z * a).atan2(p * b);
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let lat = (self.z + e2 * b * sin_theta.powi(3)).atan2(p - e2 * a * cos_theta.powi(3));
        let lon = self.y.atan2(self.x);
        let elevation = p / cos_theta - a / ((1.0 - e2 * sin_theta.powi(2)).sqrt());
        LlePoint::new(lat.to_degrees(), lon.to_degrees(), elevation)
    }
    pub fn to_enu(&self, ref_point: &LlePoint) -> EnuPoint {
        let lat_ref = ref_point.lat().to_radians();
        let h_ref = ref_point.elevation();

        let a = phys_const::EARTH_SEMI_MAJOR_AXIS;
        let b = phys_const::EARTH_SEMI_MINOR_AXIS;
        let e2 = (a * a - b * b) / (a * a);
        let n = a / ((1.0 - e2 * lat_ref.sin().powi(2)).sqrt());
        let x_ref = (n + h_ref) * lat_ref.cos() * lat_ref.cos();
        let y_ref = (n + h_ref) * lat_ref.cos() * lat_ref.sin();
        let z_ref = ((1.0 - e2) * n + h_ref) * lat_ref.sin();

        let dx = self.x - x_ref;
        let dy = self.y - y_ref;
        let dz = self.z - z_ref;

        let e = -lat_ref.sin() * dx + lat_ref.cos() * dy;
        let n = -lat_ref.cos() * lat_ref.sin() * dx - lat_ref.sin() * lat_ref.sin() * dy + lat_ref.cos() * dz;
        let u = lat_ref.cos() * lat_ref.cos() * dx + lat_ref.sin() * lat_ref.cos() * dy + lat_ref.sin() * dz;

        EnuPoint::new(e, n, u)
    }
}

impl EnuPoint {
    pub fn new(e: f64, n: f64, u: f64) -> Self {
        EnuPoint { e, n, u }
    }

    pub fn e(&self) -> f64 {
        self.e
    }

    pub fn n(&self) -> f64 {
        self.n
    }

    pub fn u(&self) -> f64 {
        self.u
    }

    pub fn to_ecef(&self, ref_point: &LlePoint) -> EcefPoint {
        let lat_ref = ref_point.lat().to_radians();
        let h_ref = ref_point.elevation();

        let a = phys_const::EARTH_SEMI_MAJOR_AXIS;
        let b = phys_const::EARTH_SEMI_MINOR_AXIS;
        let e2 = (a * a - b * b) / (a * a);
        let n = a / ((1.0 - e2 * lat_ref.sin().powi(2)).sqrt());
        let x_ref = (n + h_ref) * lat_ref.cos() * lat_ref.cos();
        let y_ref = (n + h_ref) * lat_ref.cos() * lat_ref.sin();
        let z_ref = ((1.0 - e2) * n + h_ref) * lat_ref.sin();

        let dx = self.e * -lat_ref.sin() + self.n * -lat_ref.cos() * lat_ref.sin() + self.u * lat_ref.cos() * lat_ref.cos();
        let dy = self.e * lat_ref.cos() + self.n * -lat_ref.sin() * lat_ref.sin() + self.u * lat_ref.sin() * lat_ref.cos();
        let dz = self.n * lat_ref.cos() + self.u * lat_ref.sin();

        EcefPoint::new(x_ref + dx, y_ref + dy, z_ref + dz)
    }
    pub fn to_lle(&self, ref_point: &LlePoint) -> LlePoint {
        let ecef = self.to_ecef(ref_point);
        ecef.to_lle()
    }
}

impl LlePoint {
    pub fn new(lat: f64, lon: f64, elevation: f64) -> Self {
        LlePoint { lat, lon, elevation }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn elevation(&self) -> f64 {
        self.elevation
    }

    pub fn to_ecef(&self) -> EcefPoint {
        let lat_rad = self.lat.to_radians();
        let lon_rad = self.lon.to_radians();
        let a = phys_const::EARTH_SEMI_MAJOR_AXIS;
        let b = phys_const::EARTH_SEMI_MINOR_AXIS;
        let e2 = (a * a - b * b) / (a * a);
        let n = a / ((1.0 - e2 * lat_rad.sin().powi(2)).sqrt());
        let x = (n + self.elevation) * lat_rad.cos() * lon_rad.cos();
        let y = (n + self.elevation) * lat_rad.cos() * lon_rad.sin();
        let z = ((1.0 - e2) * n + self.elevation) * lat_rad.sin();
        EcefPoint::new(x, y, z)
    }
    pub fn to_enu(&self, ref_point: &LlePoint) -> EnuPoint {
        let ecef = self.to_ecef();
        ecef.to_enu(ref_point)
    }
}
