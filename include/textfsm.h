#ifndef TEXTFSM_H
#define TEXTFSM_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Parse device output using a TextFSM template.
 *
 * All arguments must be valid, null-terminated C strings.
 * Returns a JSON envelope (null-terminated) that the caller must free
 * with textfsm_free().
 */
const char *textfsm_parse_json(const char *vendor,
                               const char *command_key,
                               const char *template_text,
                               const char *output_text);

/**
 * Free a string previously returned by textfsm_parse_json().
 * Passing NULL is safe (no-op).
 */
void textfsm_free(const char *s);

#ifdef __cplusplus
}
#endif

#endif /* TEXTFSM_H */
