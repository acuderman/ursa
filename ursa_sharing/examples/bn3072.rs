// Copyright 2020 Hyperledger Ursa Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use ursa_sharing::{error::*, tests::*, Field, Group};

use generic_array::{typenum::U384, GenericArray};
use openssl::bn::{BigNum, BigNumContext};
use rand::{CryptoRng, RngCore};

struct Bn3072(BigNum);

/// Safe prime generated by OpenSSL
const MODULUS: &str= "2810648864918553692326414703540982236400161676089967413931080487336351664151951596142800952137837786949602780093696245504389122898007840882257958695803988348815090801641319246142910064654295656659531928562021941244973691275871456332653319544240941768988728044605685210468374028296470195771445893463172468596886296397514478881750748854380716952364198170004560459348379205872353140560333434262518706391341981993534904655587887145767376775247180777074746668503322468682710283049559264203107937893133977707072588631910572571691340287270684887753691017850014026114991899690570003338568598941699411176562161168853436661856010103788770425973503571121826245383549444250163796809725074965822253566072469306063871256250293519396973490875540895189911205333942452223362768239991669324715576323296472026485762324469371989244243246598338329148439857434726497797195444870550503807350157997765978494536147484675781002402359451435754948869927";

impl Field for Bn3072 {
    fn one() -> Self {
        Self(BigNum::from_u32(1).unwrap())
    }

    fn from_usize(value: usize) -> Self {
        Self(BigNum::from_dec_str(value.to_string().as_str()).unwrap())
    }

    fn scalar_div_assign(&mut self, rhs: &Self) {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let r = BigNum::from_slice(self.0.to_vec().as_slice()).unwrap();
        let mut h = BigNum::new().unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        h.mod_inverse(&rhs.0, &n, &mut ctx).unwrap();
        self.0.mod_mul(&r, &h, &n, &mut ctx).unwrap();
    }
}

impl Group for Bn3072 {
    type Size = U384;

    fn zero() -> Self {
        Self(BigNum::new().unwrap())
    }

    fn from_bytes<B: AsRef<[u8]>>(value: B) -> SharingResult<Self> {
        let bn = BigNum::from_slice(value.as_ref()).unwrap();
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        if bn < n {
            Ok(Self(bn))
        } else {
            Err(SharingError::ShareInvalidSecret)
        }
    }

    fn random(_: &mut (impl RngCore + CryptoRng)) -> Self {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let mut r = BigNum::new().unwrap();
        n.rand_range(&mut r).unwrap();
        Self(r)
    }

    fn is_zero(&self) -> bool {
        self.0 == BigNum::new().unwrap()
    }

    fn is_valid(&self) -> bool {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        !self.is_zero() && self.0 < n
    }

    fn negate(&mut self) {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let z = BigNum::new().unwrap();
        let r = BigNum::from_slice(self.0.to_vec().as_slice()).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        self.0.mod_sub(&z, &r, &n, &mut ctx).unwrap();
    }

    fn add_assign(&mut self, rhs: &Self) {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let r = BigNum::from_slice(self.0.to_vec().as_slice()).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        self.0.mod_add(&r, &rhs.0, &n, &mut ctx).unwrap();
    }

    fn sub_assign(&mut self, rhs: &Self) {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let r = BigNum::from_slice(self.0.to_vec().as_slice()).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        self.0.mod_sub(&r, &rhs.0, &n, &mut ctx).unwrap();
    }

    fn scalar_mul_assign(&mut self, rhs: &Bn3072) {
        let n = BigNum::from_dec_str(MODULUS).unwrap();
        let r = BigNum::from_slice(self.0.to_vec().as_slice()).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        self.0.mod_mul(&r, &rhs.0, &n, &mut ctx).unwrap();
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let mut v = self.0.to_vec();
        let mut o = vec![0u8; 384 - v.len()];
        o.append(&mut v);
        GenericArray::clone_from_slice(o.as_slice())
    }
}

fn main() {
    println!("Splitting");
    split_invalid_args::<Bn3072>();
    println!("Combine invalid fail");
    combine_invalid::<Bn3072>();
    println!("Combine single success");
    combine_single::<Bn3072, Bn3072>();
    println!("Combine combinations success");
    combine_all_combinations::<Bn3072, Bn3072>();
}
