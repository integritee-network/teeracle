/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use rustls::{internal::pemfile::rsa_private_keys, PrivateKey};
use std::io::BufReader;

pub fn get_test_private_key() -> PrivateKey {
	let mut buf_reader = BufReader::new(PRIVATE_KEY_STR.as_bytes());
	rsa_private_keys(&mut buf_reader).unwrap().first().unwrap().clone()
}

const PRIVATE_KEY_STR: &str = "\
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAmuJ6JwsQKxEvpAQX/ZmaGkNhgTIO+D49r3uESiUiAbfQOxIC
eIRc5xUNdTtdV7293VUBjivqRUcYdsYNAyb1d5Scrabynpr4sk+Skhr6eh8m6pse
ugdld4g3614YckqhubShHnMxQSKvutnQKdPq9KEnaWShk9WA7V6J43G5w2/dyu4P
GqxGnRSmDdzAIwRkN5uX7FiSbquIS1NfnaZgH8ZmB9XZXJ/PS8j4WIdCO0eb9e1a
xpTQliFg9Xpi7TPL00veHjLafEejLjCGUzioAU6yKVOBuRj3Ed35Q88y7AsRLbY4
zGuNc0rSl84nlCMIq0QMz7+PFeCZZAglD7/e6QIDAQABAoIBAQCEe5i08Nehnw+7
Ie1LdSnFsUEj+6emW8bz5ZlguqZ+BbbN8DfA0qeM2gsq7d6IALr5KY8tBw9atteM
MRhMS/THloz2VMlPNYvpKftbkkwSTbdCEfGUemMmfZQnddM/X+s6J/FxVGMbLgpW
r51JSgW9vmMx2WwEQioH4EfeDxcwvZi3LF7SAo89eMSiSDqHZaIfMRmS0cSpoXav
u7gKDt7H+zSeYdLC4FhD4f8zRUpZEa4x5GIIm2JHsvIWuy9XKyepakaObJkWWqR1
ATO94LtM2+RRVUev+yOVDDOfJtDzEqZrbokCHaVBYXgliAV/XkvFox1ZINyeGFq4
kAvqfiQJAoGBAMhO/tAz2TpWeETMcujBekx1JmtDEUITJroDT0DvFDV5QRKVopxY
ZY5pPbwtk60KknBbsXrswR3Vh1q3xfKLT3Ln4x121ufltIwN7eopY9dXVqh830CU
QymtUz5VcvG3foWCeABcyklpZIdhHyDDDDP46URfFr3NnQiRnx7qb6yPAoGBAMXy
bSGgnBPUOWHtNW4hI5vxiOiCGWvCq7jERixybGMU8+kP6eRWUEAnOdCibq84A6gv
GLO5EW+bmL8l7L797w6ZN9DhbuR7W7hQVwdkyQS8PUgmTfsaba7+9hTC0chl+L38
A7NlYRju+JS99SqarGA6WMvo30ykiMGwxw8tHOkHAoGAPT6Z/oK72nBx2WdBgxUV
FaeEFaut7Sv53UoBw3LWFPt7//isfW0xr/dRnuW4j2H6IEyI2XLmIP8WoZAq/9vE
cPeho3KghsrfByuDIOOC2Wak4mM7x30NhAKwvxBVUr6t+phHpKS6XPPSfuodIGFC
q+lhOTxxsZradrI/mq5HctUCgYEAqo4bYeIVGTC+0JWmd+Gt4OvYXx3Z8XOmqmjT
XfCpWyXuk13W1ZtZQi2KLy4F2IuW+w65ZgGL+HJExk5TEq2RkS6LXTsgZVW0zbbL
hd9dJOtckhIPFtDKuQGN3o2OW/EgxfGi7qvnYahmHyMdXzwuUitz3x4jaNJL0zgS
DA1+33kCgYA1iAZ58XXJPh6YObvw+kg21dCLLelxp+mCoRBSbY6wq+R6PmKg4a1N
oOc6Rh/1teyBVWJ/KnkXBeh9//XLfhg0r6zHDSCsDKabeM0eoB1AKWlc5f6bWYHV
60JHDgby+V1AElKT2yQT8KVv1hWJH4XQ1/fTQpQDDoo6O+nj1r4q6w==
-----END RSA PRIVATE KEY-----";
