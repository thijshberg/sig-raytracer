# Info

A raytracing signal propagation modeler. 
Based on https://github.com/dps/rust-raytracer.

The code heavily benefits from more cores to run.

# How to run

Basic usage: ``cargo run <config_file> <output_file>''
See /data/ for a sample config file

Result is written to <output_file>.data

## Optional flags
These flags are useful for debugging purposes
### --angles 
Create file containing the angles with which the ray that hit each point arrived.
### --times
Create file containing the travel time of the ray that hit each point
### --png
Render the signal map as a png
### --view
Render the camera-angle picture. Useful for debugging the placement logic and collision.

