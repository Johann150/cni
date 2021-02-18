//! The traits in this module supply the API that the specification recommends.
//!
//! The [`Cni`] trait implementations provide [`SubTree`] and [`SubLeaves`].
//!
//! The functions `ListTree` and `ListLeaves` may be produced by using e.g.
//! [`HashMap::values`] on the results of the [`SubTree`] and [`SubLeaves`] functions.
//!
//! The [`CniIter`] trait implementations provide the [`WalkTree`] and
//! [`WalkLeaves`] functions.
//!
//! The function names are provided with the Rust naming convention and are
//! aliased with more descriptive names.
//!
//! [`Cni`]: trait.Cni.html
//! [`SubTree`]: Cni::in_section
//! [`SubLeaves`]: Cni::children_in_section
//! [`CniIter`]: trait.CniIter.html
//! [`WalkTree`]: CniIter::in_section
//! [`WalkLeaves`]: CniIter::children_in_section
//! [`HashMap::values`]: ::std::collections::HashMap::values

use std::cell::RefCell;
use std::iter::FromIterator;

/// Provides the [`SubTree`] and [`SubLeaves`] functions.
///
/// You can use the blanket implementations for this trait by importing it.
///
/// [`SubTree`]: Cni::in_section
/// [`SubLeaves`]: Cni::children_in_section
pub trait Cni: Sized {
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubTree`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni::api::Cni;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let parsed = cni::parse(&cni).expect("could not parse CNI");
    ///
    /// let mut result = HashMap::new();
    /// result.insert("key".to_string(), "value".to_string());
    /// result.insert("subsection.key".to_string(), "other value".to_string());
    ///
    /// assert_eq!(parsed.in_section("section"), result);
    /// ```
    ///
    /// Use e.g. [`HashMap::values`] to get `ListTree`.
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    fn in_section(&self, section: &str) -> Self;
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubLeaves`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni::api::Cni;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let parsed = cni::parse(&cni).expect("could not parse CNI");
    ///
    /// let mut result = HashMap::new();
    /// result.insert("key".to_string(), "value".to_string());
    ///
    /// assert_eq!(parsed.children_in_section("section"), result);
    /// ```
    /// Use e.g. [`HashMap::values`] to get `ListLeaves`.
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    fn children_in_section(&self, section: &str) -> Self;
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// This is an alias for [`Cni::in_section`].
    /// The CNI specification calls this `SubTree`.
    ///
    /// Use e.g. [`HashMap::values`] to get `ListTree`.
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    fn sub_tree(&self, section: &str) -> Self {
        self.in_section(section)
    }
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// This is an alias for [`Cni::children_in_section`].
    /// The CNI specification calls this `SubLeaves`.
    ///
    /// Use e.g. [`HashMap::values`] to get `ListLeaves`.
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    fn sub_leaves(&self, section: &str) -> Self {
        self.children_in_section(section)
    }
}

impl<I, K, V> Cni for I
where
    I: IntoIterator<Item = (K, V)> + Clone + FromIterator<(String, V)>,
    K: AsRef<str>,
    V: Clone,
{
    /// Implements the `SubTree` API function.
    fn in_section(&self, section: &str) -> Self {
        self.clone()
            .into_iter()
            .filter_map(|(k, v)| {
                let k = k.as_ref();
                if k.starts_with(section) && k[section.len()..].starts_with('.') {
                    Some((k[section.len() + 1..].to_string(), v))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Implements the `SubLeaves` API function.
    fn children_in_section(&self, section: &str) -> Self {
        self.clone()
            .into_iter()
            .filter_map(|(k, v)| {
                let k = k.as_ref();
                if k.starts_with(section)
                    && k[section.len()..].starts_with('.')
                    && !k[section.len() + 1..].contains('.')
                {
                    Some((k[section.len() + 1..].to_string(), v))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Provides the [`WalkTree`] and [`WalkLeaves`] functions.
/// There are blanket implementations for appropriate Iterators.
///
/// [`WalkTree`]: CniIter::in_section
/// [`WalkLeaves`]: CniIter::children_in_section
pub trait CniIter: Sized {
    /// Returns an iterator that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// The CNI specification calls this `WalkTree`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni::api::CniIter;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut parsed = cni::parse(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     .in_section("section")
    ///     // have to clone here because we want to store the result
    ///     .map(|(k, v)| (k.clone(), v.clone()))
    ///     .collect::<Vec<_>>();
    /// // because the order is unspecified, have to sort to compare
    /// parsed.sort();
    ///
    /// assert_eq!(
    ///     parsed,
    ///     vec![
    ///         ("section.key".to_string(), "value".to_string()),
    ///         ("section.subsection.key".to_string(), "other value".to_string()),
    ///     ]
    /// );
    /// ```
    fn in_section(self, section: &str) -> SectionFilter<Self>;
    /// Returns an iterator that only contains direct child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// The CNI specification calls this `WalkLeaves`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni::api::CniIter;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut parsed = cni::parse(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     .children_in_section("section")
    ///     // have to clone here because we want to store the result
    ///     .map(|(k, v)| (k.clone(), v.clone()))
    ///     .collect::<Vec<_>>();
    /// // because the order is unspecified, have to sort to compare
    /// parsed.sort();
    ///
    /// assert_eq!(
    ///     parsed,
    ///     vec![
    ///         ("section.key".to_string(), "value".to_string()),
    ///     ]
    /// );
    /// ```
    fn children_in_section(self, section: &str) -> SectionFilter<Self>;
    /// Returns an iterator that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// This is an alias for [`CniIter::in_section`].
    /// The CNI specification calls this `WalkTree`.
    fn walk_tree(self, section: &str) -> SectionFilter<Self> {
        self.in_section(section)
    }
    /// Returns an iterator that only contains direct child elements of the
    /// specified section. The section name and delimiter will be included
    /// the result. The order is unspecified.
    ///
    /// This is an alias for [`CniIter::children_in_section`].
    /// The CNI specification calls this `WalkLeaves`.
    fn walk_leaves(self, section: &str) -> SectionFilter<Self> {
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
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.borrow_mut().find(|(k, _)| {
            let k = k.as_ref();
            k.starts_with(self.section)
                && k[self.section.len()..].starts_with('.')
                && !(self.only_direct_children && k[self.section.len() + 1..].contains('.'))
        })
    }
}

impl<I: Iterator> CniIter for I {
    /// Implements the `WalkTree` API function.
    fn in_section<'section>(self, section: &str) -> SectionFilter<Self> {
        SectionFilter {
            iter: RefCell::new(self),
            section,
            only_direct_children: false,
        }
    }

    /// Implements the `WalkLeaves` API function.
    fn children_in_section<'section>(self, section: &str) -> SectionFilter<Self> {
        SectionFilter {
            iter: RefCell::new(self),
            section,
            only_direct_children: true,
        }
    }
}
