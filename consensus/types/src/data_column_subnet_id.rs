//! Identifies each data column subnet by an integer identifier.
use crate::data_column_sidecar::ColumnIndex;
use crate::EthSpec;
use ethereum_types::U256;
use itertools::Itertools;
use safe_arith::{ArithError, SafeArith};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::{self, Display};
use std::ops::{Deref, DerefMut};

const DATA_COLUMN_SUBNET_COUNT: u64 = 32;

lazy_static! {
    static ref DATA_COLUMN_SUBNET_ID_TO_STRING: Vec<String> = {
        let mut v = Vec::with_capacity(DATA_COLUMN_SUBNET_COUNT as usize);

        for i in 0..DATA_COLUMN_SUBNET_COUNT {
            v.push(i.to_string());
        }
        v
    };
}

#[derive(arbitrary::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DataColumnSubnetId(#[serde(with = "serde_utils::quoted_u64")] u64);

pub fn data_column_subnet_id_to_string(i: u64) -> &'static str {
    if i < DATA_COLUMN_SUBNET_COUNT {
        DATA_COLUMN_SUBNET_ID_TO_STRING
            .get(i as usize)
            .expect("index below DATA_COLUMN_SUBNET_COUNT")
    } else {
        "data column subnet id out of range"
    }
}

impl DataColumnSubnetId {
    pub fn new(id: u64) -> Self {
        id.into()
    }

    pub fn from_column_index<E: EthSpec>(column_index: usize) -> Self {
        (column_index
            .safe_rem(E::data_column_subnet_count())
            .expect("data_column_subnet_count should never be zero if this function is called")
            as u64)
            .into()
    }

    #[allow(clippy::arithmetic_side_effects)]
    pub fn columns<E: EthSpec>(&self) -> impl Iterator<Item = ColumnIndex> {
        let subnet = self.0;
        let data_column_subnet_count = E::data_column_subnet_count() as u64;
        let columns_per_subnet = E::data_columns_per_subnet() as u64;
        (0..columns_per_subnet).map(move |i| data_column_subnet_count * i + subnet)
    }

    /// Compute required subnets to subscribe to given the node id.
    /// TODO(das): Add epoch param
    #[allow(clippy::arithmetic_side_effects)]
    pub fn compute_custody_subnets<E: EthSpec>(
        node_id: U256,
        custody_subnet_count: u64,
    ) -> impl Iterator<Item = DataColumnSubnetId> {
        // TODO(das): we could perform check on `custody_subnet_count` here to ensure that it is a valid
        // value, but here we assume it is valid.

        let mut subnets = SmallVec::<[u64; 32]>::new();
        let mut current_id = node_id;
        while (subnets.len() as u64) < custody_subnet_count {
            let mut node_id_bytes = [0u8; 32];
            current_id.to_little_endian(&mut node_id_bytes);
            let hash = ethereum_hashing::hash_fixed(&node_id_bytes);
            let hash_prefix = [
                hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
            ];
            let hash_prefix_u64 = u64::from_le_bytes(hash_prefix);
            let subnet = hash_prefix_u64 % (E::data_column_subnet_count() as u64);

            if !subnets.contains(&subnet) {
                subnets.push(subnet);
            }

            if current_id == U256::MAX {
                current_id = U256::zero()
            }
            current_id += U256::one()
        }
        subnets.into_iter().map(DataColumnSubnetId::new)
    }

    pub fn compute_custody_columns<E: EthSpec>(
        node_id: U256,
        custody_subnet_count: u64,
    ) -> impl Iterator<Item = ColumnIndex> {
        Self::compute_custody_subnets::<E>(node_id, custody_subnet_count)
            .flat_map(|subnet| subnet.columns::<E>())
            .sorted()
    }
}

impl Display for DataColumnSubnetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Deref for DataColumnSubnetId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataColumnSubnetId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u64> for DataColumnSubnetId {
    fn from(x: u64) -> Self {
        Self(x)
    }
}

impl Into<u64> for DataColumnSubnetId {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<u64> for &DataColumnSubnetId {
    fn into(self) -> u64 {
        self.0
    }
}

impl AsRef<str> for DataColumnSubnetId {
    fn as_ref(&self) -> &str {
        data_column_subnet_id_to_string(self.0)
    }
}

#[derive(Debug)]
pub enum Error {
    ArithError(ArithError),
}

impl From<ArithError> for Error {
    fn from(e: ArithError) -> Self {
        Error::ArithError(e)
    }
}

#[cfg(test)]
mod test {
    use crate::data_column_subnet_id::DataColumnSubnetId;
    use crate::EthSpec;
    use crate::MainnetEthSpec;

    type E = MainnetEthSpec;

    #[test]
    fn test_compute_subnets_for_data_column() {
        let node_ids = [
            "0",
            "88752428858350697756262172400162263450541348766581994718383409852729519486397",
            "18732750322395381632951253735273868184515463718109267674920115648614659369468",
            "27726842142488109545414954493849224833670205008410190955613662332153332462900",
            "39755236029158558527862903296867805548949739810920318269566095185775868999998",
            "31899136003441886988955119620035330314647133604576220223892254902004850516297",
            "58579998103852084482416614330746509727562027284701078483890722833654510444626",
            "28248042035542126088870192155378394518950310811868093527036637864276176517397",
            "60930578857433095740782970114409273483106482059893286066493409689627770333527",
            "103822458477361691467064888613019442068586830412598673713899771287914656699997",
        ]
        .into_iter()
        .map(|v| ethereum_types::U256::from_dec_str(v).unwrap())
        .collect::<Vec<_>>();

        let custody_requirement = 4;
        for node_id in node_ids {
            let computed_subnets =
                DataColumnSubnetId::compute_custody_subnets::<E>(node_id, custody_requirement);
            let computed_subnets: Vec<_> = computed_subnets.collect();

            // the number of subnets is equal to the custody requirement
            assert_eq!(computed_subnets.len() as u64, custody_requirement);

            let subnet_count = E::data_column_subnet_count();
            for subnet in computed_subnets {
                let columns: Vec<_> = subnet.columns::<E>().collect();
                // the number of columns is equal to the specified number of columns per subnet
                assert_eq!(columns.len(), E::data_columns_per_subnet());

                for pair in columns.windows(2) {
                    // each successive column index is offset by the number of subnets
                    assert_eq!(pair[1] - pair[0], subnet_count as u64);
                }
            }
        }
    }
}