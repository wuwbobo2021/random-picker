use crate::*;
use std::{collections::HashMap, fmt::Display, hash::Hash, str::FromStr};

/// Alias of `HashMap`. The weight value type is always `f64`.
pub type Table<T> = HashMap<T, f64, std::hash::RandomState>;

/// Configuration required by `Picker`. All members are public
/// and are supposed to be modified by the user.
#[derive(Clone, PartialEq, Debug)]
pub struct Config<T: Clone + Eq + Hash> {
    /// Table of choices and weights which are proportional to the probabilities
    /// on repetitive mode or single-item mode.
    pub table: Table<T>,

    /// Do multiplicative inversion for each value in the table.
    pub inversed: bool,

    /// Allow the same item to be picked for multiple times in the result.
    pub repetitive: bool,
}

impl<T: Clone + Eq + Hash> Config<T> {
    /// Returns an invalid configuration with an empty table.
    /// Please add items into the table before using it to construct `Picker`.
    #[inline]
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            inversed: false,
            repetitive: false,
        }
    }

    /// Checks whether or not the table can be used by `Picker`.
    pub fn check(&self) -> Result<(), Error> {
        if self.table.is_empty() {
            return Err(Error::InvalidTable);
        }
        for &v in self.table.values() {
            if v < 0. || (self.inversed && v == 0.) {
                return Err(Error::InvalidTable);
            }
        }
        Ok(())
    }

    /// Returns `true` if all items have equal (and valid) weight values.
    pub fn is_fair(&self) -> bool {
        if self.check().is_err() {
            return false;
        }
        let mut v_prev = None;
        for &v in self.table.values() {
            if let Some(v_prev) = v_prev {
                if v != v_prev {
                    return false;
                }
            }
            v_prev.replace(v);
        }
        true
    }

    #[inline]
    pub(crate) fn vec_table(&self) -> Result<Vec<(T, f64)>, Error> {
        self.check()?;
        let vec = if !self.inversed {
            self.table.iter().map(|(k, &v)| (k.clone(), v)).collect()
        } else {
            self.table
                .iter()
                .map(|(k, &v)| (k.clone(), 1. / v))
                .collect()
        };
        Ok(vec)
    }
}

impl<T: Clone + Eq + Hash> Default for Config<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Config<String> {
    /// Appends, modifies or deletes items in the table
    /// according to the configuration input string.
    ///
    /// ```
    /// let mut conf: random_picker::Config<String> = "
    /// repetitive = true
    /// inversed = false
    /// # this line can be ignored
    /// [items]
    /// oxygen = 47
    /// sillicon = 28
    /// aluminium=8; iron=5; magnesium=4;
    /// calcium=2: potassium=2: sodium=2:
    /// others = 2; nonexistium = 31
    ///    aluminium = 9; delete nonexistium
    /// ".parse().unwrap();
    /// assert_eq!(conf.table.len(), 9);
    /// assert_eq!(conf.repetitive, true);
    /// assert_eq!(conf.inversed, false);
    /// assert_eq!(conf.table.get("aluminium"), Some(&9.));
    ///
    /// conf.append_str("\
    /// power_inversed
    /// ## invalid: repetitive = 0
    /// repetitive = 0
    /// delete others
    /// ");
    /// assert_eq!(conf.repetitive, true);
    /// assert_eq!(conf.inversed, true);
    ///
    /// conf.append_str("inversed = false");
    /// assert_eq!(conf.inversed, false);
    /// ```
    pub fn append_str(&mut self, str_items: &str) {
        for line in str_items.split(&['\r', '\n', ';', ':']) {
            let mut spl = line.split(&[' ', '\t', '=']).filter(|s| !s.is_empty());
            let item_name;
            if let Some(s) = spl.next() {
                if s.chars().nth(0) == Some('#') {
                    continue;
                }
                item_name = s;
            } else {
                continue;
            }

            // compatible with the old table format
            if item_name == "power_inversed" {
                self.inversed = true;
                continue;
            }
            if item_name == "repetitive_picking" {
                self.repetitive = true;
                continue;
            }

            if let Some(s) = spl.last() {
                if item_name == "delete" {
                    let _ = self.table.remove(s);
                } else if let Ok(b) = bool::from_str(s) {
                    if item_name == "inversed" {
                        self.inversed = b;
                    } else if item_name == "repetitive" {
                        self.repetitive = b;
                    }
                } else if let Ok(v) = f64::from_str(s) {
                    self.table.insert(item_name.to_string(), v);
                }
            }
        }
    }
}

impl FromStr for Config<String> {
    type Err = Error;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut conf = Self::new();
        conf.append_str(s);
        Ok(conf)
    }
}

impl Display for Config<String> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.check().is_err() {
            writeln!(f, "# INVALID!!!")?;
        }
        writeln!(f, "[random-picker]")?;
        writeln!(f, "repetitive = {}", self.repetitive)?;
        writeln!(f, "inversed = {}\n", self.inversed)?;
        writeln!(f, "[items]")?;
        format_table(f, &self.table)?;
        Ok(())
    }
}

/// Prints the weight table to the standard output.
#[inline(always)]
pub fn print_table(table: &Table<String>) {
    let mut s = String::new();
    let _ = format_table(&mut s, table);
    print!("{s}");
}

fn format_table(f: &mut impl std::fmt::Write, table: &Table<String>) -> std::fmt::Result {
    let name_len_max;
    if let Some(n) = table.keys().map(|s| s.len()).max() {
        name_len_max = n;
    } else {
        // empty?
        return Ok(());
    }

    let mut vec_table: Vec<_> = table.iter().collect();
    vec_table.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    for (k, v) in vec_table {
        writeln!(f, "{:>2$} = {:>9.6}", k, v, name_len_max)?;
    }
    Ok(())
}
