impl GuiHost {
    // C++ lifecycle matrix alignment (`cmd_object.cpp`): when an object slot is reused by create*
    // the new generation must not inherit terminal movie state from previous generation.
    // Therefore reset-for-create always clears both failure and interrupted snapshots.
    fn reset_object_runtime_state_for_create(&mut self, plane: StagePlane, object_index: i32) {
        let st = self.get_or_create_object_state(plane, object_index);
        reset_object_state_preserve_seq(st);
        self.movie_playing_objects.remove(&(plane, object_index));
        self.movie_ready_objects.remove(&(plane, object_index));
        self.movie_generations.remove(&(plane, object_index));
        self.clear_movie_terminal_state(plane, object_index);
        self.clear_object_string_state(plane, object_index);
        self.clear_object_string_style_state(plane, object_index);
        self.clear_object_number_state(plane, object_index);
        self.clear_object_number_style_state(plane, object_index);
        self.clear_object_button_state(plane, object_index);
        self.clear_object_weather_state(plane, object_index);
        self.clear_object_movie_seek_state(plane, object_index);
    }
}
