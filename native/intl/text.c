#include "bridge.h"

#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unicode/uloc.h>
#include <unicode/ustring.h>
#include <unicode/uversion.h>

int32_t wow_icu_fail(WowIcuError *error, UErrorCode code,
                     const char *operation) {
  error->code = (int32_t)code;
  error->operation = operation;
  return WOW_ICU_ERROR;
}

UBool wow_icu_preflight(UErrorCode status, WowIcuError *error,
                        const char *operation) {
  if (status == U_BUFFER_OVERFLOW_ERROR || U_SUCCESS(status)) {
    return 1;
  }
  wow_icu_fail(error, status, operation);
  return 0;
}

void *wow_icu_allocate(int32_t length, size_t width, WowIcuError *error) {
  if (length < 0 || length == INT32_MAX || width == 0 ||
      (size_t)length + 1 > SIZE_MAX / width) {
    wow_icu_fail(error, U_INDEX_OUTOFBOUNDS_ERROR,
                 "allocate conversion buffer");
    return NULL;
  }
  void *buffer = calloc((size_t)length + 1, width);
  if (buffer == NULL) {
    wow_icu_fail(error, U_MEMORY_ALLOCATION_ERROR,
                 "allocate conversion buffer");
  }
  return buffer;
}

UChar *wow_icu_to_utf16(const uint8_t *text, int32_t length,
                        int32_t *out_length, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = 0;
  u_strFromUTF8(NULL, 0, &required, (const char *)text, length, &status);
  if (!wow_icu_preflight(status, error, "size UTF-16 input"))
    return NULL;
  UChar *buffer = wow_icu_allocate(required, sizeof(UChar), error);
  if (buffer == NULL)
    return NULL;
  status = U_ZERO_ERROR;
  u_strFromUTF8(buffer, required + 1, out_length, (const char *)text, length,
                &status);
  if (U_FAILURE(status)) {
    free(buffer);
    wow_icu_fail(error, status, "convert UTF-8 input");
    return NULL;
  }
  return buffer;
}

UBool wow_icu_to_utf8(const UChar *text, int32_t length, WowIcuString *output,
                      WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = 0;
  u_strToUTF8(NULL, 0, &required, text, length, &status);
  if (!wow_icu_preflight(status, error, "size UTF-8 output"))
    return 0;
  uint8_t *buffer = wow_icu_allocate(required, sizeof(uint8_t), error);
  if (buffer == NULL)
    return 0;
  status = U_ZERO_ERROR;
  u_strToUTF8((char *)buffer, required + 1, &output->length, text, length,
              &status);
  if (U_FAILURE(status)) {
    free(buffer);
    wow_icu_fail(error, status, "convert UTF-16 output");
    return 0;
  }
  output->data = buffer;
  return 1;
}

/* ICU underscore identifiers are an explicit input form, not a failed-tag
 * fallback. */
static char *language_tag(const char *locale, int32_t length,
                          WowIcuError *error) {
  if (memchr(locale, '_', (size_t)length) == NULL) {
    char *copy = wow_icu_allocate(length, sizeof(char), error);
    if (copy != NULL)
      memcpy(copy, locale, (size_t)length);
    return copy;
  }
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = uloc_toLanguageTag(locale, NULL, 0, 1, &status);
  if (!wow_icu_preflight(status, error, "size ICU locale language tag"))
    return NULL;
  char *tag = wow_icu_allocate(required, sizeof(char), error);
  if (tag == NULL)
    return NULL;
  status = U_ZERO_ERROR;
  uloc_toLanguageTag(locale, tag, required + 1, 1, &status);
  if (U_FAILURE(status)) {
    free(tag);
    wow_icu_fail(error, status, "convert ICU locale to language tag");
    return NULL;
  }
  return tag;
}

static char *parse_language_tag(const char *tag, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t parsed = 0;
  int32_t required = uloc_forLanguageTag(tag, NULL, 0, &parsed, &status);
  if (!wow_icu_preflight(status, error, "size parsed locale"))
    return NULL;
  /* tag is a checked, allocated locale identifier, not numeric input text. */
  if ((size_t)parsed != strlen(tag)) {
    wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR,
                 "consume complete locale language tag");
    return NULL;
  }
  char *locale = wow_icu_allocate(required, sizeof(char), error);
  if (locale == NULL)
    return NULL;
  status = U_ZERO_ERROR;
  uloc_forLanguageTag(tag, locale, required + 1, &parsed, &status);
  if (U_FAILURE(status)) {
    free(locale);
    wow_icu_fail(error, status, "parse locale language tag");
    return NULL;
  }
  return locale;
}

char *wow_icu_locale(const char *locale, int32_t length, WowIcuError *error) {
  char *tag = language_tag(locale, length, error);
  if (tag == NULL)
    return NULL;
  char *parsed = parse_language_tag(tag, error);
  free(tag);
  return parsed;
}

void wow_icu_string_free(uint8_t *data) { free(data); }

const char *wow_icu_error_name(int32_t code) {
  return u_errorName((UErrorCode)code);
}

void wow_icu_version(uint8_t version[4]) { u_getVersion(version); }
