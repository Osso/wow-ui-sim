#include "bridge.h"

#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unicode/uloc.h>
#include <unicode/utrans.h>

static int32_t display_name(const char *target, const char *display,
                            WowIcuString *output, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = uloc_getDisplayName(target, display, NULL, 0, &status);
  if (!wow_icu_preflight(status, error, "size locale display name"))
    return WOW_ICU_ERROR;
  UChar *buffer = wow_icu_allocate(required, sizeof(UChar), error);
  if (buffer == NULL)
    return WOW_ICU_ERROR;
  status = U_ZERO_ERROR;
  int32_t length = uloc_getDisplayName(target, display, buffer, required + 1,
                                      &status);
  int32_t result;
  if (U_FAILURE(status))
    result = wow_icu_fail(error, status, "get locale display name");
  else
    result = wow_icu_to_utf8(buffer, length, output, error)
                 ? WOW_ICU_OK : WOW_ICU_ERROR;
  free(buffer);
  return result;
}

int32_t wow_icu_display_name(const char *target, int32_t target_length,
                              const char *display, int32_t display_length,
                              WowIcuString *output, WowIcuError *error) {
  char *target_id = wow_icu_locale(target, target_length, error);
  if (target_id == NULL)
    return WOW_ICU_ERROR;
  char *display_id = wow_icu_locale(display, display_length, error);
  if (display_id == NULL) {
    free(target_id);
    return WOW_ICU_ERROR;
  }
  int32_t result = display_name(target_id, display_id, output, error);
  free(display_id);
  free(target_id);
  return result;
}

static UTransliterator *open_transliterator(const uint8_t *id, int32_t length,
                                           WowIcuError *error) {
  if (length == 0 || memchr(id, 0, (size_t)length) != NULL) {
    wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "validate transliterator ID");
    return NULL;
  }
  int32_t utf16_length = 0;
  UChar *utf16 = wow_icu_to_utf16(id, length, &utf16_length, error);
  if (utf16 == NULL)
    return NULL;
  UErrorCode status = U_ZERO_ERROR;
  UTransliterator *trans = utrans_openU(utf16, utf16_length, UTRANS_FORWARD,
                                       NULL, 0, NULL, &status);
  free(utf16);
  if (U_FAILURE(status) || trans == NULL) {
    if (trans != NULL)
      utrans_close(trans);
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open transliterator");
    return NULL;
  }
  return trans;
}

static int32_t grow_capacity(int32_t capacity, int32_t required,
                              WowIcuError *error) {
  int64_t next = (int64_t)capacity * 2;
  if ((int64_t)required + 1 > next)
    next = (int64_t)required + 1;
  if (next >= INT32_MAX) {
    wow_icu_fail(error, U_INDEX_OUTOFBOUNDS_ERROR, "grow transliteration buffer");
    return 0;
  }
  return (int32_t)next;
}

static int32_t transliterate_original(const UTransliterator *trans,
                                      const UChar *original, int32_t length,
                                      WowIcuString *output, WowIcuError *error) {
  int32_t capacity = length + 1;
  for (;;) {
    UChar *buffer = wow_icu_allocate(capacity - 1, sizeof(UChar), error);
    if (buffer == NULL)
      return WOW_ICU_ERROR;
    /* Overflow may leave partially transformed text. Every attempt starts from
     * the untouched input and resets both in/out length and limit. */
    memcpy(buffer, original, (size_t)length * sizeof(UChar));
    int32_t transformed_length = length;
    int32_t limit = length;
    UErrorCode status = U_ZERO_ERROR;
    utrans_transUChars(trans, buffer, &transformed_length, capacity, 0, &limit,
                       &status);
    if (status == U_BUFFER_OVERFLOW_ERROR) {
      free(buffer);
      capacity = grow_capacity(capacity, transformed_length, error);
      if (capacity == 0)
        return WOW_ICU_ERROR;
      continue;
    }
    int32_t result;
    if (U_FAILURE(status))
      result = wow_icu_fail(error, status, "transliterate text");
    else
      result = wow_icu_to_utf8(buffer, transformed_length, output, error)
                   ? WOW_ICU_OK : WOW_ICU_ERROR;
    free(buffer);
    return result;
  }
}

int32_t wow_icu_transliterate(const uint8_t *text, int32_t text_length,
                               const uint8_t *id, int32_t id_length,
                               WowIcuString *output, WowIcuError *error) {
  UTransliterator *trans = open_transliterator(id, id_length, error);
  if (trans == NULL)
    return WOW_ICU_ERROR;
  int32_t length = 0;
  UChar *original = wow_icu_to_utf16(text, text_length, &length, error);
  if (original == NULL) {
    utrans_close(trans);
    return WOW_ICU_ERROR;
  }
  int32_t result = transliterate_original(trans, original, length, output, error);
  free(original);
  utrans_close(trans);
  return result;
}
