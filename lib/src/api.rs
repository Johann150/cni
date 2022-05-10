//! This module supplies the API that the CNI specification recommends.
//!
//! The function names are provided with the Rust naming convention.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::iter::FromIterator;

/// Provides the recommended API functions:
/// * [`SubTree`] and [`SubLeaves`]
///     * `ListTree` and `ListLeaves` by using e.g. [`HashMap::values`] on this
///     * `KeyTree` and `KeyLeaves` by using e.g. [`HashMap::keys`] on this
/// * [`WalkTree`] and [`WalkLeaves`]
/// * [`SectionTree`] and [`SectionLeaves`]
///
/// You can use the blanket implementations for this trait by importing it.
///
/// [`SubTree`]: CniExt::sub_tree
/// [`SubLeaves`]: CniExt::sub_leaves
/// [`WalkTree`]: CniExt::walk_tree
/// [`WalkLeaves`]: CniExt::walk_leaves
/// [`SectionTree`]: CniExt::section_tree
/// [`SectionLeaves`]: CniExt::section_leaves
/// [`HashMap::values`]: ::std::collections::HashMap::values
/// [`HashMap::keys`]: ::std::collections::HashMap::keys
pub trait CniExt<V>: Sized {
    /// The type of the underlying iterator produced by some functions.
    type Iter;
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
    /// use cni_format::CniExt;
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
    #[must_use]
    fn sub_tree(&self, section: &str) -> Self
    where
        Self: Clone + FromIterator<(String, V)>;
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
    /// use cni_format::CniExt;
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
    #[must_use]
    fn sub_leaves(&self, section: &str) -> Self
    where
        Self: Clone + FromIterator<(String, V)>;
    /// Returns an iterator that only contains child elements of the
    /// specified section. The section name and delimiter will be included in
    /// the result. The order is unspecified.
    ///
    /// The CNI specification calls this `WalkTree`.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::CniExt;
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
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::CniExt;
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
    ///     .walk_leaves("section")
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
    /// Returns the names of subsection of the specified section. Note that
    /// this does not necessarily mean that the respective section names are in
    /// the source as section headers.
    ///
    /// The CNI specification calls this `SectionTree`.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::CniExt;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut sections = cni_format::from_str(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     .section_tree("section");
    ///
    /// assert_eq!(
    ///     sections.into_iter().collect::<Vec<_>>(),
    ///     vec![
    ///         "subsection".to_string(),
    ///     ]
    /// );
    /// ```
    fn section_tree(&self, section: &str) -> BTreeSet<String>
    where
        Self: Clone;
    /// Returns the names of direct subsections of the specified section. Note
    /// that this does not necessarily mean that the respective section names
    /// are in the source as section headers.
    ///
    /// The CNI specification calls this `SectionTree`.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use cni_format::CniExt;
    ///
    /// let cni = r"
    /// [section]
    /// key = value
    /// subsection.key = other value
    /// [otherSection]
    /// key = value
    /// ";
    ///
    /// let mut sections = cni_format::from_str(&cni)
    ///     .expect("could not parse CNI")
    ///     .iter()
    ///     // get direct subsections of top level section
    ///     .section_leaves("");
    ///
    /// assert_eq!(
    ///     sections.into_iter().collect::<Vec<_>>(),
    ///     vec![
    ///         "otherSection".to_string(), "section".to_string(),
    ///     ]
    /// );
    /// ```
    fn section_leaves(&self, section: &str) -> BTreeSet<String>
    where
        Self: Clone;
}

impl<T, I, K, V> CniExt<V> for T
where
    T: IntoIterator<IntoIter = I>,
    I: Iterator<Item = (K, V)>,
    K: AsRef<str>,
{
    type Iter = I;

    /// Implements the `SubTree` API function.
    fn sub_tree(&self, section: &str) -> Self
    where
        Self: Clone + FromIterator<(String, V)>,
    {
        self.clone()
            .into_iter()
            .filter_map(|(k, v)| {
                let k = k.as_ref();
                if section.is_empty() {
                    Some((k.to_string(), v))
                } else if k.starts_with(section) && k[section.len()..].starts_with('.') {
                    Some((k[section.len() + 1..].to_string(), v))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Implements the `SubLeaves` API function.
    fn sub_leaves(&self, section: &str) -> Self
    where
        Self: Clone + FromIterator<(String, V)>,
    {
        self.clone()
            .into_iter()
            .filter_map(|(k, v)| {
                let k = k.as_ref();
                if section.is_empty() && !k.contains('.') {
                    Some((k.to_string(), v))
                } else if k.starts_with(section)
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

    /// Implements the `SectionTree` API function.
    fn section_tree(&self, section: &str) -> BTreeSet<String>
    where
        T: Clone,
    {
        // TODO: `keys` could be simplified if nightly feature map_first_last is
        // stabilized, see <https://github.com/rust-lang/rust/issues/62924>
        let mut keys = vec![];
        let mut result = BTreeSet::new();
        keys.extend(
            self.clone()
                .walk_tree(section)
                // ignore current section's name
                .map(|(k, _)| {
                    if section.is_empty() {
                        k.as_ref().to_string()
                    } else {
                        k.as_ref()[section.len() + 1..].to_string()
                    }
                }),
        );

        while let Some(key) = keys.pop() {
            if let Some(pos) = key.rfind('.') {
                let section = key.split_at(pos).0.to_string();
                if !keys.contains(&section) && !result.contains(&section) {
                    keys.push(section.clone());
                }
                result.insert(section.to_string());
            }
        }

        result
    }

    fn section_leaves(&self, section: &str) -> BTreeSet<String>
    where
        T: Clone,
    {
        // TODO: `keys` could be simplified if nightly feature map_first_last is
        // stabilized, see <https://github.com/rust-lang/rust/issues/62924>
        let mut keys = vec![];
        let mut result = BTreeSet::new();
        keys.extend(
            self.clone()
                .walk_tree(section)
                // ignore current section's name
                .map(|(k, _)| {
                    if section.is_empty() {
                        k.as_ref().to_string()
                    } else {
                        k.as_ref()[section.len() + 1..].to_string()
                    }
                }),
        );

        while let Some(key) = keys.pop() {
            if let Some(pos) = key.rfind('.') {
                let section = key.split_at(pos).0.to_string();
                if section.contains('.') {
                    continue;
                }
                if !keys.contains(&section) && !result.contains(&section) {
                    keys.push(section.clone());
                }
                result.insert(section.to_string());
            }
        }

        result
    }
}

/// An iterator that filters the elements of a key-value iterator for keys in
/// a specific section.
///
/// This `struct` is created by the [`walk_tree`]  and [`walk_leaves`]
/// methods on [`CniExt`]. See its documentation for more.
///
/// [`walk_tree`]: CniExt::walk_tree
/// [`walk_leaves`]: CniExt::walk_leaves
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
                && (k[self.section.len()..].starts_with('.') || self.section.is_empty())
                && !(self.only_direct_children && k[self.section.len() + 1..].contains('.'))
        })
    }
}
