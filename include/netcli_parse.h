#ifndef NETCLI_PARSE_H
#define NETCLI_PARSE_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Parse network device CLI output into a structured JSON envelope.
 *
 * @param platform     Platform identifier, e.g. "cisco_ios", "junos", "dnos".
 * @param command_key  Stable command intent key, e.g. "show_version".
 * @param output_text  Raw CLI text to parse.
 *
 * @return JSON envelope (null-terminated C string). The caller MUST free the
 *         returned pointer with netcli_free().
 *
 * Success: {"ok":true,"platform":"...","commandKey":"...","records":[...]}
 * Error:   {"ok":false,"error":{"code":"...","message":"..."}}
 */
const char *netcli_parse_json(const char *platform,
                              const char *command_key,
                              const char *output_text);

/**
 * Parse network device CLI output using a raw command string.
 *
 * The command is normalized internally (spaces become underscores, lowercased)
 * to match against the template registry. Otherwise identical to
 * netcli_parse_json().
 *
 * @param platform     Platform identifier, e.g. "cisco_ios", "junos", "dnos".
 * @param command      Raw CLI command, e.g. "show version", "show ip bgp summary".
 * @param output_text  Raw CLI text to parse.
 *
 * @return JSON envelope (null-terminated C string). The caller MUST free the
 *         returned pointer with netcli_free().
 */
const char *netcli_parse_command_json(const char *platform,
                                      const char *command,
                                      const char *output_text);

/**
 * Free a string previously returned by netcli_parse_json() or
 * netcli_parse_command_json(). Passing NULL is safe (no-op).
 */
void netcli_free(const char *s);

#ifdef __cplusplus
}
#endif

#endif /* NETCLI_PARSE_H */
