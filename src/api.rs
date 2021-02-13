//! The traits in this module supply the API that the specification recommends.
//!
//! The [`Cni`] trait implementations provide [`SubRec`] and [`SubFlat`].
//!
//! The functions `ListRec` and `ListFlat` may be produced by using
//! [`HashMap::values`] on the results of the [`SubRec`] and [`SubFlat`] functions.
//!
//! The [`CniIter`] trait implementations provide the [`WalkRec`] and
//! [`WalkFlat`] functions.
//!
//! The function names are provided with the Rust naming convention and are
//! aliased with more descriptive names.
//!
//! [`Cni`]: trait.Cni.html
//! [`SubRec`]: Cni::in_section
//! [`SubFlat`]: Cni::children_in_section
//! [`CniIter`]: trait.CniIter.html
//! [`WalkRec`]: CniIter::in_section
//! [`WalkFlat`]: CniIter::children_in_section

use std::cell::RefCell;
use std::collections::HashMap;

/// Provides the [`SubRec`] and [`SubFlat`] functions.
///
/// [`SubRec`]: Cni::in_section
/// [`SubFlat`]: Cni::children_in_section
pub trait Cni: Sized {
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubRec`.
    ///
    /// Use [`HashMap::values`] to get `ListRec`.
    fn in_section(&self, section: &str) -> Self;
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubFlat`.
    ///
    /// Use [`HashMap::values`] to get `ListFlat`.
    fn children_in_section(&self, section: &str) -> Self;
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// This is an alias for `in_section`.
    /// The CNI specification calls this `SubRec`.
    ///
    /// Use [`HashMap::values`] to get `ListRec`.
    fn sub_rec(&self, section: &str) -> Self {
        self.in_section(section)
    }
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// This is an alias for `children_in_section`.
    /// The CNI specification calls this `SubFlat`.
    ///
    /// Use [`HashMap::values`][] to get `ListFlat`.
    fn sub_flat(&self, section: &str) -> Self {
        self.children_in_section(section)
    }
}

impl<T: Clone> Cni for HashMap<String, T> {
    fn in_section(&self, section: &str) -> Self {
        self.iter()
            .filter_map(|(k, v)| {
                if k.starts_with(section)
                	&& k[section.len()..].starts_with('.')
                {
                    Some((k[section.len() + 1..].to_string(), v.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn children_in_section(&self, section: &str) -> Self {
        self.iter()
            .filter_map(|(k, v)| {
                if k.starts_with(section)
                	&& k[section.len()..].starts_with('.')
                	&& !k[section.len()+1..].contains('.')
                {
                    Some((k[section.len() + 1..].to_string(), v.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Provides the [`WalkRec`] and [`WalkFlat`] functions.
///
/// [`WalkRec`]: CniIter::in_section
/// [`WalkFlat`]: CniIter::children_in_section
pub trait CniIter: Sized {
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result.
    ///
    /// The CNI specification calls this `WalkRec`.
    fn in_section(self, section: &str) -> SectionFilter<Self>;
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result.
    ///
    /// The CNI specification calls this `WalkFlat`.
    fn children_in_section(self, section: &str) -> SectionFilter<Self>;
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result.
    ///
    /// This is an alias for `in_section`.
    /// The CNI specification calls this `WalkRec`.
    fn walk_rec(self, section: &str) -> SectionFilter<Self> {
        self.in_section(section)
    }
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be included
    /// the result.
    ///
    /// This is an alias for `children_in_section`.
    /// The CNI specification calls this `WalkFlat`.
    fn walk_flat(self, section: &str) -> SectionFilter<Self> {
        self.children_in_section(section)
    }
}

/// An iterator that filters the elements of a key-value iterator for keys in
/// a specific section.
///
/// This `struct` is created by the [`in_section`]  and [`children_in_section`]
/// methods on [`CniIter`]. See its documentation for more.
///
/// [`in_section`]: CniIter::in_section
/// [`children_in_section`]: CniIter::children_in_section
/// [`CniIter`]: trait.CniIter.html
pub struct SectionFilter<'section, I> {
    iter: RefCell<I>,
    section: &'section str,
    only_direct_children: bool,
}

impl<I, K, V> Iterator for SectionFilter<'_, I>
where
    I: Iterator<Item = (K, V)>,
    K: AsRef<str>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
    	self.iter.borrow_mut().find(|(k, _v)| {
    		let k = k.as_ref();
            k.starts_with(self.section)
            && k[self.section.len()..].starts_with('.')
            && !(self.only_direct_children && k[self.section.len() + 1..].contains('.'))
        })
    }
}

impl<I: Iterator> CniIter for I {
    fn in_section<'section>(self, section: &str) -> SectionFilter<Self> {
        SectionFilter {
            iter: RefCell::new(self),
            section,
            only_direct_children: false,
        }
    }

    fn children_in_section<'section>(self, section: &str) -> SectionFilter<Self> {
        SectionFilter {
            iter: RefCell::new(self),
            section,
            only_direct_children: true,
        }
    }
}
