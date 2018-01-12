extern crate hackattic;
use hackattic::HackatticChallenge;

fn main() {
    hackattic::face_detect::FaceDetection::process_challenge().unwrap();
}