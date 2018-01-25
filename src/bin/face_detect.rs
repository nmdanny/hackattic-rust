extern crate hackattic;
use hackattic::HackatticChallenge;


fn main() {
    #[cfg(facedetect)]
        {
            hackattic::face_detect::FaceDetection::process_challenge().unwrap();
        };
    #[cfg(not(facedetect))]
        {
            panic!("Can't run face_detect without the 'facedetect' feature enabled!(requires opencv)");
        };
}
