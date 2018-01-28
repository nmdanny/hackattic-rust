extern crate hackattic;
use hackattic::HackatticChallenge;

#[cfg(feature = "facedetect")]
fn main() {
	hackattic::face_detect::FaceDetection::process_challenge().unwrap();
}

#[cfg(not(feature = "facedetect"))]
fn main() {
	panic!("Can't run face_detect without the 'facedetect' feature enabled!(requires opencv)");
}
