#include "bridge.h"

#include <stdlib.h>
#include <unicode/ucurr.h>

/* Metadata calls otherwise return generic defaults for unknown currency codes.
 * Query ICU's catalogue across all dates before accepting either result. Rust
 * validates and uppercases the three ASCII letters before entering this ABI. */
static int32_t known_currency(const char *currency, UChar code[4],
                              WowIcuError *error) {
  for (int32_t i = 0; i < 3; ++i)
    code[i] = (UChar)(uint8_t)currency[i];
  code[3] = 0;
  UErrorCode status = U_ZERO_ERROR;
  UBool known = ucurr_isAvailable(code, U_DATE_MIN, U_DATE_MAX, &status);
  if (U_FAILURE(status))
    return wow_icu_fail(error, status, "query currency catalogue");
  return known ? WOW_ICU_OK : WOW_ICU_NO_MATCH;
}

/* WoW's Long and NarrowSymbol ordering differs from ICU's enum ordering. */
static int32_t read_name_style(int32_t style, UCurrNameStyle *result,
                               WowIcuError *error) {
  switch (style) {
  case 0: *result = UCURR_SYMBOL_NAME; break;
  case 1: *result = UCURR_NARROW_SYMBOL_NAME; break;
  case 2: *result = UCURR_LONG_NAME; break;
  case 3: *result = UCURR_FORMAL_SYMBOL_NAME; break;
  case 4: *result = UCURR_VARIANT_SYMBOL_NAME; break;
  default:
    return wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "select currency name style");
  }
  return WOW_ICU_OK;
}

int32_t wow_icu_currency_name(const char *locale, int32_t locale_length,
                              const char *currency, int32_t style,
                              WowIcuString *output, WowIcuError *error) {
  UCurrNameStyle name_style;
  int32_t result = read_name_style(style, &name_style, error);
  if (result != WOW_ICU_OK)
    return result;
  UChar code[4];
  result = known_currency(currency, code, error);
  if (result != WOW_ICU_OK)
    return result;
  char *parsed_locale = wow_icu_locale(locale, locale_length, error);
  if (parsed_locale == NULL)
    return WOW_ICU_ERROR;
  UErrorCode status = U_ZERO_ERROR;
  UBool is_choice = 0;
  int32_t length = 0;
  const UChar *name = ucurr_getName(code, parsed_locale, name_style,
                                   &is_choice, &length, &status);
  free(parsed_locale);
  if (U_FAILURE(status))
    return wow_icu_fail(error, status, "read currency name");
  if (name == NULL)
    return wow_icu_fail(error, U_INTERNAL_PROGRAM_ERROR, "read currency name");
  /* ICU owns name; only the copied UTF-8 output is returned to Rust. */
  return wow_icu_to_utf8(name, length, output, error) ? WOW_ICU_OK : WOW_ICU_ERROR;
}

int32_t wow_icu_currency_fraction_digits(const char *currency, int32_t *digits,
                                         WowIcuError *error) {
  UChar code[4];
  int32_t result = known_currency(currency, code, error);
  if (result != WOW_ICU_OK)
    return result;
  UErrorCode status = U_ZERO_ERROR;
  int32_t value = ucurr_getDefaultFractionDigits(code, &status);
  if (U_FAILURE(status))
    return wow_icu_fail(error, status, "read currency fraction digits");
  *digits = value;
  return WOW_ICU_OK;
}
