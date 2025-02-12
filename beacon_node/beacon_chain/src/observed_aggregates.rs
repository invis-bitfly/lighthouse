//! Provides an `ObservedAggregates` struct which allows us to reject aggregated attestations or
//! sync committee contributions if we've already seen them.

use crate::sync_committee_verification::SyncCommitteeData;
use ssz_types::{BitList, BitVector};
use std::collections::HashMap;
use std::marker::PhantomData;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;
use types::consts::altair::{
    SYNC_COMMITTEE_SUBNET_COUNT, TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE,
};
use types::slot_data::SlotData;
use types::{
    Attestation, AttestationData, AttestationRef, EthSpec, Hash256, Slot, SyncCommitteeContribution,
};

pub type ObservedSyncContributions<E> = ObservedAggregates<
    SyncCommitteeContribution<E>,
    E,
    BitVector<<E as types::EthSpec>::SyncSubcommitteeSize>,
>;
pub type ObservedAggregateAttestations<E> =
    ObservedAggregates<Attestation<E>, E, BitList<<E as types::EthSpec>::MaxValidatorsPerSlot>>;

/// Attestation data augmented with committee index
///
/// This is hashed and used to key the map of observed aggregate attestations. This is important
/// post-Electra where the attestation data committee index is 0 and we want to avoid accidentally
/// comparing aggregation bits for *different* committees.
#[derive(TreeHash)]
pub struct ObservedAttestationKey {
    pub committee_index: u64,
    pub attestation_data: AttestationData,
}

/// A trait use to associate capacity constants with the type being stored in `ObservedAggregates`.
pub trait Consts {
    /// The default capacity of items stored per slot, in a single `SlotHashSet`.
    const DEFAULT_PER_SLOT_CAPACITY: usize;

    /// The maximum number of slots
    fn max_slot_capacity() -> usize;

    /// The maximum number of items stored per slot, in a single `SlotHashSet`.
    fn max_per_slot_capacity() -> usize;
}

impl<E: EthSpec> Consts for Attestation<E> {
    /// Use 128 as it's the target committee size for the mainnet spec. This is perhaps a little
    /// wasteful for the minimal spec, but considering it's approx. 128 * 32 bytes we're not wasting
    /// much.
    const DEFAULT_PER_SLOT_CAPACITY: usize = 128;

    /// We need to keep attestations for each slot of the current epoch.
    fn max_slot_capacity() -> usize {
        2 * E::slots_per_epoch() as usize
    }

    /// As a DoS protection measure, the maximum number of distinct `Attestations` or
    /// `SyncCommitteeContributions` that will be recorded for each slot.
    ///
    /// Currently this is set to ~524k. If we say that each entry is 40 bytes (Hash256 (32 bytes) + an
    /// 8 byte hash) then this comes to about 20mb per slot. If we're storing 34 of these slots, then
    /// we're at 680mb. This is a lot of memory usage, but probably not a show-stopper for most
    /// reasonable hardware.
    ///
    /// Upstream conditions should strongly restrict the amount of attestations that can show up in
    /// this pool. The maximum size with respect to upstream restrictions is more likely on the order
    /// of the number of validators.
    fn max_per_slot_capacity() -> usize {
        1 << 19 // 524,288
    }
}

impl<E: EthSpec> Consts for SyncCommitteeContribution<E> {
    /// Set to `TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE * SYNC_COMMITTEE_SUBNET_COUNT`. This is the
    /// expected number of aggregators per slot across all subcommittees.
    const DEFAULT_PER_SLOT_CAPACITY: usize =
        (SYNC_COMMITTEE_SUBNET_COUNT * TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE) as usize;

    /// We only need to keep contributions related to the current slot.
    fn max_slot_capacity() -> usize {
        1
    }

    /// We should never receive more aggregates than there are sync committee participants.
    fn max_per_slot_capacity() -> usize {
        E::sync_committee_size()
    }
}

/// A trait for types that implement a behaviour where one object of that type
/// can be a subset/superset of another.
/// This trait allows us to be generic over the aggregate item that we store in the cache that
/// we want to prevent duplicates/subsets for.
pub trait SubsetItem {
    /// The item that is stored for later comparison with new incoming aggregate items.
    type Item;

    /// Returns `true` if `self` is a non-strict subset of `other` and `false` otherwise.
    fn is_subset(&self, other: &Self::Item) -> bool;

    /// Returns `true` if `self` is a non-strict superset of `other` and `false` otherwise.
    fn is_superset(&self, other: &Self::Item) -> bool;

    /// Returns the item that gets stored in `ObservedAggregates` for later subset
    /// comparison with incoming aggregates.
    fn get_item(&self) -> Result<Self::Item, Error>;

    /// Returns a unique value that keys the object to the item that is being stored
    /// in `ObservedAggregates`.
    fn root(&self) -> Result<Hash256, Error>;
}

impl<'a, E: EthSpec> SubsetItem for AttestationRef<'a, E> {
    type Item = BitList<E::MaxValidatorsPerSlot>;
    fn is_subset(&self, other: &Self::Item) -> bool {
        match self {
            Self::Base(att) => {
                if let Ok(extended_aggregation_bits) = att.extend_aggregation_bits() {
                    return extended_aggregation_bits.is_subset(other);
                }
                false
            }
            Self::Electra(att) => att.aggregation_bits.is_subset(other),
        }
    }

    fn is_superset(&self, other: &Self::Item) -> bool {
        match self {
            Self::Base(att) => {
                if let Ok(extended_aggregation_bits) = att.extend_aggregation_bits() {
                    return other.is_subset(&extended_aggregation_bits);
                }
                false
            }
            Self::Electra(att) => other.is_subset(&att.aggregation_bits),
        }
    }

    /// Returns the sync contribution aggregation bits.
    fn get_item(&self) -> Result<Self::Item, Error> {
        match self {
            Self::Base(att) => att
                .extend_aggregation_bits()
                .map_err(|_| Error::GetItemError),
            Self::Electra(att) => Ok(att.aggregation_bits.clone()),
        }
    }

    /// Returns the hash tree root of the attestation data augmented with the committee index.
    fn root(&self) -> Result<Hash256, Error> {
        Ok(ObservedAttestationKey {
            committee_index: self.committee_index().ok_or(Error::RootError)?,
            attestation_data: self.data().clone(),
        }
        .tree_hash_root())
    }
}

impl<'a, E: EthSpec> SubsetItem for &'a SyncCommitteeContribution<E> {
    type Item = BitVector<E::SyncSubcommitteeSize>;
    fn is_subset(&self, other: &Self::Item) -> bool {
        self.aggregation_bits.is_subset(other)
    }

    fn is_superset(&self, other: &Self::Item) -> bool {
        other.is_subset(&self.aggregation_bits)
    }

    /// Returns the sync contribution aggregation bits.
    fn get_item(&self) -> Result<Self::Item, Error> {
        Ok(self.aggregation_bits.clone())
    }

    /// Returns the hash tree root of the root, slot and subcommittee index
    /// of the sync contribution.
    fn root(&self) -> Result<Hash256, Error> {
        Ok(SyncCommitteeData {
            root: self.beacon_block_root,
            slot: self.slot,
            subcommittee_index: self.subcommittee_index,
        }
        .tree_hash_root())
    }
}

#[derive(Debug, PartialEq)]
pub enum ObserveOutcome {
    /// This item is a non-strict subset of an already known item.
    Subset,
    /// This was the first time this item was observed.
    New,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    SlotTooLow {
        slot: Slot,
        lowest_permissible_slot: Slot,
    },
    /// The function to obtain a set index failed, this is an internal error.
    InvalidSetIndex(usize),
    /// We have reached the maximum number of unique items that can be observed in a slot.
    /// This is a DoS protection function.
    ReachedMaxObservationsPerSlot(usize),
    IncorrectSlot {
        expected: Slot,
        attestation: Slot,
    },
    GetItemError,
    RootError,
}

/// A `HashMap` that contains entries related to some `Slot`.
struct SlotHashSet<I> {
    /// Contains a vector of maximally-sized aggregation bitfields/bitvectors
    /// such that no bitfield/bitvector is a subset of any other in the list.
    map: HashMap<Hash256, Vec<I>>,
    slot: Slot,
    max_capacity: usize,
}

impl<I> SlotHashSet<I> {
    pub fn new(slot: Slot, initial_capacity: usize, max_capacity: usize) -> Self {
        Self {
            slot,
            map: HashMap::with_capacity(initial_capacity),
            max_capacity,
        }
    }

    /// Store the items in self so future observations recognise its existence.
    pub fn observe_item<S: SlotData + SubsetItem<Item = I>>(
        &mut self,
        item: S,
        root: Hash256,
    ) -> Result<ObserveOutcome, Error> {
        if item.get_slot() != self.slot {
            return Err(Error::IncorrectSlot {
                expected: self.slot,
                attestation: item.get_slot(),
            });
        }

        if let Some(aggregates) = self.map.get_mut(&root) {
            for existing in aggregates {
                // Check if `item` is a subset of any of the observed aggregates
                if item.is_subset(existing) {
                    return Ok(ObserveOutcome::Subset);
                // Check if `item` is a superset of any of the observed aggregates
                // If true, we replace the new item with its existing subset. This allows us
                // to hold fewer items in the list.
                } else if item.is_superset(existing) {
                    *existing = item.get_item()?;
                    return Ok(ObserveOutcome::New);
                }
            }
        }

        // Here we check to see if this slot has reached the maximum observation count.
        //
        // The resulting behaviour is that we are no longer able to successfully observe new
        // items, however we will continue to return `is_known_subset` values. We could also
        // disable `is_known_subset`, however then we would stop forwarding items across the
        // gossip network and I think that this is a worse case than sending some invalid ones.
        // The underlying libp2p network is responsible for removing duplicate messages, so
        // this doesn't risk a broadcast loop.
        if self.map.len() >= self.max_capacity {
            return Err(Error::ReachedMaxObservationsPerSlot(self.max_capacity));
        }

        let item = item.get_item()?;
        self.map.entry(root).or_default().push(item);
        Ok(ObserveOutcome::New)
    }

    /// Check if `item` is a non-strict subset of any of the already observed aggregates for
    /// the given root and slot.
    pub fn is_known_subset<S: SlotData + SubsetItem<Item = I>>(
        &self,
        item: S,
        root: Hash256,
    ) -> Result<bool, Error> {
        if item.get_slot() != self.slot {
            return Err(Error::IncorrectSlot {
                expected: self.slot,
                attestation: item.get_slot(),
            });
        }

        Ok(self
            .map
            .get(&root)
            .map_or(false, |agg| agg.iter().any(|val| item.is_subset(val))))
    }

    /// The number of observed items in `self`.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

/// Trait for observable items that can be observed from their reference type.
///
/// This is used to make observations for `Attestation`s from `AttestationRef`s.
pub trait AsReference {
    type Reference<'a>
    where
        Self: 'a;

    fn as_reference(&self) -> Self::Reference<'_>;
}

impl<E: EthSpec> AsReference for Attestation<E> {
    type Reference<'a> = AttestationRef<'a, E>;

    fn as_reference(&self) -> AttestationRef<'_, E> {
        self.to_ref()
    }
}

impl<E: EthSpec> AsReference for SyncCommitteeContribution<E> {
    type Reference<'a> = &'a Self;

    fn as_reference(&self) -> &Self {
        self
    }
}

/// Stores the roots of objects for some number of `Slots`, so we can determine if
/// these have previously been seen on the network.
pub struct ObservedAggregates<T: Consts + AsReference, E: EthSpec, I> {
    lowest_permissible_slot: Slot,
    sets: Vec<SlotHashSet<I>>,
    _phantom_spec: PhantomData<E>,
    _phantom_tree_hash: PhantomData<T>,
}

impl<T: Consts + AsReference, E: EthSpec, I> Default for ObservedAggregates<T, E, I> {
    fn default() -> Self {
        Self {
            lowest_permissible_slot: Slot::new(0),
            sets: vec![],
            _phantom_spec: PhantomData,
            _phantom_tree_hash: PhantomData,
        }
    }
}

impl<T, E, I> ObservedAggregates<T, E, I>
where
    T: Consts + AsReference,
    E: EthSpec,
    for<'a> T::Reference<'a>: SubsetItem<Item = I> + SlotData,
{
    /// Store `item` in `self` keyed at `root`.
    ///
    /// `root` must equal `item.root::<SubsetItem>()`.
    pub fn observe_item(
        &mut self,
        item: T::Reference<'_>,
        root_opt: Option<Hash256>,
    ) -> Result<ObserveOutcome, Error> {
        let index = self.get_set_index(item.get_slot())?;
        let root = root_opt.map_or_else(|| item.root(), Ok)?;

        self.sets
            .get_mut(index)
            .ok_or(Error::InvalidSetIndex(index))
            .and_then(|set| set.observe_item(item, root))
    }

    /// Check if `item` is a non-strict subset of any of the already observed aggregates for
    /// the given root and slot.
    ///
    /// `root` must equal `item.root::<SubsetItem>()`.
    #[allow(clippy::wrong_self_convention)]
    pub fn is_known_subset(
        &mut self,
        item: T::Reference<'_>,
        root: Hash256,
    ) -> Result<bool, Error> {
        let index = self.get_set_index(item.get_slot())?;

        self.sets
            .get(index)
            .ok_or(Error::InvalidSetIndex(index))
            .and_then(|set| set.is_known_subset(item, root))
    }

    /// The maximum number of slots that items are stored for.
    fn max_capacity(&self) -> u64 {
        // We add `2` in order to account for one slot either side of the range due to
        // `MAXIMUM_GOSSIP_CLOCK_DISPARITY`.
        (T::max_slot_capacity() + 2) as u64
    }

    /// Removes any items with a slot lower than `current_slot` and bars any future
    /// item with a slot lower than `current_slot - SLOTS_RETAINED`.
    pub fn prune(&mut self, current_slot: Slot) {
        let lowest_permissible_slot = current_slot.saturating_sub(self.max_capacity() - 1);

        self.sets.retain(|set| set.slot >= lowest_permissible_slot);

        self.lowest_permissible_slot = lowest_permissible_slot;
    }

    /// Returns the index of `self.set` that matches `slot`.
    ///
    /// If there is no existing set for this slot one will be created. If `self.sets.len() >=
    /// Self::max_capacity()`, the set with the lowest slot will be replaced.
    fn get_set_index(&mut self, slot: Slot) -> Result<usize, Error> {
        let lowest_permissible_slot = self.lowest_permissible_slot;

        if slot < lowest_permissible_slot {
            return Err(Error::SlotTooLow {
                slot,
                lowest_permissible_slot,
            });
        }

        // Prune the pool if this item indicates that the current slot has advanced.
        if lowest_permissible_slot + self.max_capacity() < slot + 1 {
            self.prune(slot)
        }

        if let Some(index) = self.sets.iter().position(|set| set.slot == slot) {
            return Ok(index);
        }

        // To avoid re-allocations, try and determine a rough initial capacity for the new set
        // by obtaining the mean size of all items in earlier epoch.
        let (count, sum) = self
            .sets
            .iter()
            // Only include slots that are less than the given slot in the average. This should
            // generally avoid including recent slots that are still "filling up".
            .filter(|set| set.slot < slot)
            .map(|set| set.len())
            .fold((0, 0), |(count, sum), len| (count + 1, sum + len));
        // If we are unable to determine an average, just use the `self.default_per_slot_capacity`.
        let initial_capacity = sum
            .checked_div(count)
            .unwrap_or(T::DEFAULT_PER_SLOT_CAPACITY);

        if self.sets.len() < self.max_capacity() as usize || self.sets.is_empty() {
            let index = self.sets.len();
            self.sets.push(SlotHashSet::new(
                slot,
                initial_capacity,
                T::max_per_slot_capacity(),
            ));
            return Ok(index);
        }

        let index = self
            .sets
            .iter()
            .enumerate()
            .min_by_key(|(_i, set)| set.slot)
            .map(|(i, _set)| i)
            .expect("sets cannot be empty due to previous .is_empty() check");

        self.sets[index] = SlotHashSet::new(slot, initial_capacity, T::max_per_slot_capacity());

        Ok(index)
    }
}

#[cfg(test)]
#[cfg(not(debug_assertions))]
mod tests {
    use super::*;
    use types::{test_utils::test_random_instance, AttestationBase, Hash256};

    type E = types::MainnetEthSpec;

    fn get_attestation(slot: Slot, beacon_block_root: u64) -> Attestation<E> {
        let a: AttestationBase<E> = test_random_instance();
        let mut a = Attestation::Base(a);
        a.data_mut().slot = slot;
        a.data_mut().beacon_block_root = Hash256::from_low_u64_be(beacon_block_root);
        a
    }

    fn get_sync_contribution(slot: Slot, beacon_block_root: u64) -> SyncCommitteeContribution<E> {
        let mut a: SyncCommitteeContribution<E> = test_random_instance();
        a.slot = slot;
        a.beacon_block_root = Hash256::from_low_u64_be(beacon_block_root);
        a
    }

    macro_rules! test_suite {
        ($mod_name: ident, $type: ident, $method_name: ident) => {
            #[cfg(test)]
            mod $mod_name {
                use super::*;

                const NUM_ELEMENTS: usize = 8;

                fn single_slot_test(store: &mut $type<E>, slot: Slot) {
                    let items = (0..NUM_ELEMENTS as u64)
                        .map(|i| $method_name(slot, i))
                        .collect::<Vec<_>>();

                    for a in &items {
                        assert_eq!(
                            store.is_known_subset(
                                a.as_reference(),
                                a.as_reference().root().unwrap()
                            ),
                            Ok(false),
                            "should indicate an unknown attestation is unknown"
                        );
                        assert_eq!(
                            store.observe_item(a.as_reference(), None),
                            Ok(ObserveOutcome::New),
                            "should observe new attestation"
                        );
                    }

                    for a in &items {
                        assert_eq!(
                            store.is_known_subset(
                                a.as_reference(),
                                a.as_reference().root().unwrap()
                            ),
                            Ok(true),
                            "should indicate a known attestation is known"
                        );
                        assert_eq!(
                            store.observe_item(
                                a.as_reference(),
                                Some(a.as_reference().root().unwrap())
                            ),
                            Ok(ObserveOutcome::Subset),
                            "should acknowledge an existing attestation"
                        );
                    }
                }

                #[test]
                fn single_slot() {
                    let mut store = $type::default();

                    single_slot_test(&mut store, Slot::new(0));

                    assert_eq!(store.sets.len(), 1, "should have a single set stored");
                    assert_eq!(
                        store.sets[0].len(),
                        NUM_ELEMENTS,
                        "set should have NUM_ELEMENTS elements"
                    );
                }

                #[test]
                fn mulitple_contiguous_slots() {
                    let mut store = $type::default();
                    let max_cap = store.max_capacity();

                    for i in 0..max_cap * 3 {
                        let slot = Slot::new(i);

                        single_slot_test(&mut store, slot);

                        /*
                         * Ensure that the number of sets is correct.
                         */

                        if i < max_cap {
                            assert_eq!(
                                store.sets.len(),
                                i as usize + 1,
                                "should have a {} sets stored",
                                i + 1
                            );
                        } else {
                            assert_eq!(
                                store.sets.len(),
                                max_cap as usize,
                                "should have max_capacity sets stored"
                            );
                        }

                        /*
                         * Ensure that each set contains the correct number of elements.
                         */

                        for set in &store.sets[..] {
                            assert_eq!(
                                set.len(),
                                NUM_ELEMENTS,
                                "each store should have NUM_ELEMENTS elements"
                            )
                        }

                        /*
                         *  Ensure that all the sets have the expected slots
                         */

                        let mut store_slots =
                            store.sets.iter().map(|set| set.slot).collect::<Vec<_>>();

                        assert!(
                            store_slots.len() <= store.max_capacity() as usize,
                            "store size should not exceed max"
                        );

                        store_slots.sort_unstable();

                        let expected_slots = (i.saturating_sub(max_cap - 1)..=i)
                            .map(Slot::new)
                            .collect::<Vec<_>>();

                        assert_eq!(expected_slots, store_slots, "should have expected slots");
                    }
                }

                #[test]
                fn mulitple_non_contiguous_slots() {
                    let mut store = $type::default();
                    let max_cap = store.max_capacity();

                    let to_skip = vec![1_u64, 2, 3, 5, 6, 29, 30, 31, 32, 64];
                    let slots = (0..max_cap * 3)
                        .into_iter()
                        .filter(|i| !to_skip.contains(i))
                        .collect::<Vec<_>>();

                    for &i in &slots {
                        if to_skip.contains(&i) {
                            continue;
                        }

                        let slot = Slot::from(i);

                        single_slot_test(&mut store, slot);

                        /*
                         * Ensure that each set contains the correct number of elements.
                         */

                        for set in &store.sets[..] {
                            assert_eq!(
                                set.len(),
                                NUM_ELEMENTS,
                                "each store should have NUM_ELEMENTS elements"
                            )
                        }

                        /*
                         *  Ensure that all the sets have the expected slots
                         */

                        let mut store_slots =
                            store.sets.iter().map(|set| set.slot).collect::<Vec<_>>();

                        store_slots.sort_unstable();

                        assert!(
                            store_slots.len() <= store.max_capacity() as usize,
                            "store size should not exceed max"
                        );

                        let lowest = store.lowest_permissible_slot.as_u64();
                        let highest = slot.as_u64();
                        let expected_slots = (lowest..=highest)
                            .filter(|i| !to_skip.contains(i))
                            .map(Slot::new)
                            .collect::<Vec<_>>();

                        assert_eq!(
                            expected_slots,
                            &store_slots[..],
                            "should have expected slots"
                        );
                    }
                }
            }
        };
    }
    test_suite!(
        observed_sync_aggregates,
        ObservedSyncContributions,
        get_sync_contribution
    );
    test_suite!(
        observed_aggregate_attestations,
        ObservedAggregateAttestations,
        get_attestation
    );
}
