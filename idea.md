# Win-Glide

## Application to move Windows windows using the keyboard arrow keys.

* The application remains active waiting for a keyboard shortcut that triggers the functionality.

* When the keyboard shortcut is executed, a border of several pixels wide is drawn around the active window, and window movement is enabled via the keyboard arrow keys.

* The window movement will be fluid and dynamic, combining directions based on the keys pressed, accelerating while keys are held down and decelerating to a stop when released.

* Mouse movement overrides the arrows; that is, if there is mouse movement while the functionality is active, the mouse movement takes priority.

* The functionality stops after a few seconds without pressing direction keys or upon entering any other key.

* The sensitivity of the arrows will be a parameter specified by the user with a default value. In an initial stage, this value will be read from a configuration file.
