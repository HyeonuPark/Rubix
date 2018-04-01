use internship::IStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(IStr);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameLevel(IStr);

#[derive(Debug)]
pub struct NameIter {
    name: Name,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filter(IStr);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilterLevel {
    Name(NameLevel),
    Wildcard,
    WildMulti,
}

#[derive(Debug)]
pub struct FilterIter {
    filter: Filter,
    offset: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError;

impl Name {
    fn new(src: &str) -> Result<Self, ParseError> {

        if src.len() == 0 {
            return Err(ParseError);
        }

        let has_invalid_char = src.as_bytes().iter().any(|&ch| {
            ch == b'\0' || ch == b'+' || ch == b'#'
        });

        if has_invalid_char {
            return Err(ParseError);
        }

        Ok(Name(src.into()))
    }

    fn iter(&self) -> NameIter {
        NameIter {
            name: self.clone(),
            offset: 0,
        }
    }
}

impl Filter {
    fn new(src: &str) -> Result<Self, ParseError> {
        let bytes = src.as_bytes();

        if bytes.len() == 0 || bytes.last() == Some(&b'\0') {
            return Err(ParseError);
        }

        for window in bytes.windows(2) {
            match (window[0], window[1]) {
                (b'\0', _) | (b'#', _) => Err(ParseError),
                (b'/', _) | (_, b'/') => Ok(()),
                (b'+', _) | (_, b'+') | (_, b'#') => Err(ParseError),
                _ => Ok(())
            }?;
        }

        Ok(Filter(src.into()))
    }

    fn iter(&self) -> FilterIter {
        FilterIter {
            filter: self.clone(),
            offset: 0,
        }
    }
}

impl IntoIterator for Name {
    type IntoIter = NameIter;
    type Item = NameLevel;

    fn into_iter(self) -> NameIter {
        self.iter()
    }
}

impl Iterator for NameIter {
    type Item = NameLevel;

    fn next(&mut self) -> Option<NameLevel> {
        let name = self.name.0.as_bytes();
        let prev_offset = self.offset;

        if prev_offset == name.len() {
            return None;
        }

        let remains = name.split_at(prev_offset).1;

        let level = match remains.iter().enumerate().find(|&(_, &ch)| ch == b'/') {
            Some((offset, _)) => {
                let level = self.name.0.split_at(prev_offset).1.split_at(offset).0;
                self.offset = offset + 1;
                level
            }
            None => self.name.0.split_at(prev_offset).1,
        };

        Some(NameLevel(level.into()))
    }
}

impl IntoIterator for Filter {
    type IntoIter = FilterIter;
    type Item = FilterLevel;

    fn into_iter(self) -> FilterIter {
        self.iter()
    }
}

impl Iterator for FilterIter {
    type Item = FilterLevel;

    fn next(&mut self) -> Option<FilterLevel> {
        let filter = self.filter.0.as_bytes();
        let prev_offset = self.offset;

        if prev_offset == filter.len() {
            return None;
        }

        let remains = filter.split_at(prev_offset).1;

        let level = match remains.iter().enumerate().find(|&(_, &ch)| ch == b'/') {
            Some((offset, _)) => {
                let level = self.filter.0.split_at(prev_offset).1.split_at(offset).0;
                self.offset = offset + 1;
                level
            }
            None => self.filter.0.split_at(prev_offset).1,
        };

        Some(match level {
            "+" => FilterLevel::Wildcard,
            "#" => FilterLevel::WildMulti,
            other => FilterLevel::Name(NameLevel(other.into())),
        })
    }
}
