# AnimalCommunications
Summer Project on a Radxa Zero SBC to store device GPS data when sound is detected. The idea is that if an animal makes a loud enough sound, they we can interpret it as speech (for now). Then, we will track the gps data to see for movement of the animal. If a specific sound or group of sounds leads to a well defined movement pattern for any animal, we know it is a behavior or a possible form of communication we can study further.

The intention is that this will be attached as a collared device for an animal in the wild.

## Submission link:
This project was submitted at the below link to an IoT Competition centered around applications regarding nature in 2022. 

https://www.hackster.io/animalcomms/correlations-in-animal-movement-and-communication-camc-dd178d. 

The code under the submission is written in Python, which is not exactly ideal for an embedded system doing real time data processing. This repo is reworking that project into Rust because of both practicality and as a project to properly learn the language.

## Current Work
This project is currently only single threaded, and set to work with random sensor data (randomized f32 vectors). This is because as of now, development is still on my laptop. After completing the multithreaded design, then real sensor data will be read in. A basic FIR filter is implemented (moving average filter) to remove any peaks in the final data. I am still learning proper filtering techniques so this will likely be updated later.