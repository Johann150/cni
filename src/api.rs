//! The traits in this module supply the API that the specification recommends.
//!
//! The [`Cni`] trait implementations provide [`SubTree`] and [`SubLeaves`].
//!
//! The functions `ListTree` and `ListLeaves` may be produced by using e.g.
//! [`HashMap::values`] on the results of the [`SubTree`] and [`SubLeaves`] functions.
//!
//! The functions `KeyTree` and `KeyLeaves` may be produced by using e.g.
//! [`HashMap::keys`] on the result of the [`SubTree`] and [`SubLeaves`] functions.
//!
//! The [`CniIter`] trait implementations provide the [`WalkTree`] and
//! [`WalkLeaves`] functions.
//!
//! The function names are provided with the Rust naming convention and are
//! aliased with more descriptive names.
//!
//! [`Cni`]: trait.Cni.html
//! [`SubTree`]: Cni::sub_tree
//! [`SubLeaves`]: Cni::sub_leaves
//! [`CniIter`]: trait.CniIter.html
//! [`WalkTree`]: CniIter::walk_tree
//! [`WalkLeaves`]: CniIter::walk_leaves
//! [`HashMap::values`]: ::std::collections::HashMap::values
//! [`HashMap::keys`]: ::std::collections::HashMap::keys

use std::cell::RefCell;
use std::iter::FromIterator;

/// Provides the [`SubTree`] and [`SubLeaves`] functions.
///
/// You can use the blanket implementations for this trait by importing it.
///
/// [`SubTree`]: Cni::sub_tree
/// [`SubLeaves`]: Cni::sub_leaves
pub trait Cni: Sized {
    /// Returns a clone of self that only contains child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubTree`.
    ///
    /// Use e.g. [`HashMap::values`] to get `ListTree`.
    /// Use e.g. [`HashMap::keys`] to get `KeyTree`.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::api::Cni;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let parsed = cni_format::from_str(&cni).expect("could not parse CNI");
    ///
    /// let mut result = HashMap::new();
    /// result.insert("key".to_string(), "value".to_string());
    /// result.insert("subsection.key".to_string(), "other value".to_string());
    ///
    /// assert_eq!(parsed.sub_tree("section"), result);
    /// ```
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    /// [`HashMap::keys`]: ::std::collections::HashMap::keys
    fn sub_tree(&self, section: &str) -> Self;
    /// Returns a clone of self that only contains direct child elements of the
    /// specified section. The section name and delimiter will be removed in
    /// the result.
    ///
    /// The CNI specification calls this `SubLeaves`.
    /// Use e.g. [`HashMap::values`] to get `ListLeaves`.
    /// Use e.g. [`HashMap::keys`] to get `KeyLeaves`.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::api::Cni;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let parsed = cni_format::from_str(&cni).expect("could not parse CNI");
    ///
    /// let mut result = HashMap::new();
    /// result.insert("key".to_string(), "value".to_string());
    ///
    /// assert_eq!(parsed.sub_leaves("section"), result);
    /// ```
    ///
    /// [`HashMap::values`]: ::std::collections::HashMap::values
    /// [`HashMap::keys`]: ::std::collections::HashMap::keys
    fn sub_leaves(&self, section: &str) -> Self;
}

impl<I, K, V> Cni for I
where
    I: IntoIterator<Item = (K, V)> + Clone + FromIterator<(String, V)>,
    K: AsRef<str>,
    V: Clone,
{
    /// Implements the `SubTree` API function.
    fn sub_tree(&self, section: &str) -> Self {
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
    fn sub_leaves(&self, section: &str) -> Self {
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
/// [`WalkTree`]: CniIter::walk_tree
/// [`WalkLeaves`]: CniIter::walk_leaves
pub trait CniIter: Sized {
    /// The type of the underlying iterator.
    type Iter;
    /// Returns an iterator that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// The CNI specification calls this `WalkTree`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::api::CniIter;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut parsed = cni_format::from_str(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     .walk_tree("section")
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
    fn walk_tree(self, section: &str) -> SectionFilter<Self::Iter>;
    /// Returns an iterator that only contains direct child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// The CNI specification calls this `WalkLeaves`.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::api::CniIter;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut parsed = cni_format::from_str(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     .section_leaves("section")
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
    fn walk_leaves(self, section: &str) -> SectionFilter<Self::Iter>;
}

/// An iterator that filters the elements of a key-value iterator for keys in
/// a specific section.
///
/// This `struct` is created by the [`walk_tree`]  and [`walk_leaves`]
/// methods on [`CniIter`]. See its documentation for more.
///
/// [`walk_tree`]: CniIter::walk_tree
/// [`walk_leaves`]: CniIter::walk_leaves
/// [`CniIter`]: trait.CniIter.html
pub struct SectionFilter<'section, I> {
    // this has to use interior mutability because of how `next` has to be done
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
            // using self inside closure requires interior mutability on iter
            let k = k.as_ref();
            k.starts_with(self.section)
                && k[self.section.len()..].starts_with('.')
                && !(self.only_direct_children && k[self.section.len() + 1..].contains('.'))
        })
    }
}

impl<T, I, K, V> CniIter for T
where
    T: IntoIterator<IntoIter = I> + Clone,
    I: Iterator<Item = (K, V)>,
    K: AsRef<str>,
{
    type Iter = I;

    /// Implements the `WalkTree` API function.
    fn walk_tree(self, section: &str) -> SectionFilter<I> {
        SectionFilter {
            iter: RefCell::new(self.into_iter()),
            section,
            only_direct_children: false,
        }
    }

    /// Implements the `WalkLeaves` API function.
    fn walk_leaves(self, section: &str) -> SectionFilter<I> {
        SectionFilter {
            iter: RefCell::new(self.into_iter()),
            section,
            only_direct_children: true,
        }
    }
}
