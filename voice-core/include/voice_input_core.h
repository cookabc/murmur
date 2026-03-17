#ifndef VOICE_INPUT_CORE_H
#define VOICE_INPUT_CORE_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

char *voice_input_core_version(void);
bool voice_input_core_configure_tools(const char *ffmpeg_path, const char *coli_path);
char *voice_input_core_smoke_status_json(void);
void voice_input_core_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
