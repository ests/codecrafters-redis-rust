use std::{path::Path, error::Error, fs::File, io::Read};

use crate::{State, Duration};

pub fn load_from_rdb(path: &Path, state: &mut State, durations: &mut Duration) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    // let mut cursor = Cursor::new(buf);
    // let mut rdb = RdbParser::new(&mut cursor);
    // rdb.parse(state, durations)?;
    Ok(())
}

fn parse(state: &mut State, durations: &mut Duration) -> Result<(), Box<dyn Error>> {
    // let mut buf = [0; 9];
    // self.cursor.read_exact(&mut buf)?;
    // let mut cursor = Cursor::new(buf);
    // let mut rdb = RdbParser::new(&mut cursor);
    // rdb.parse(state, durations)?;
    Ok(())
}
