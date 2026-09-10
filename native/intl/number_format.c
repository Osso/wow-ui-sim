#include "bridge.h"

#include <math.h>
#include <stdlib.h>
#include <unicode/unum.h>

static UBool number_style(int32_t style, UNumberFormatStyle *result,
                          WowIcuError *error) {
  switch (style) {
  case WOW_DECIMAL:
  case WOW_INTEGER:
    *result = UNUM_DECIMAL;
    return 1;
  case WOW_PERCENT:
    *result = UNUM_PERCENT;
    return 1;
  case WOW_CURRENCY:
    *result = UNUM_CURRENCY;
    return 1;
  default:
    wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "select number style");
    return 0;
  }
}

static UNumberFormat *open_formatter(const char *locale, int32_t locale_length,
                                     int32_t style, WowIcuError *error) {
  UNumberFormatStyle icu_style;
  if (!number_style(style, &icu_style, error))
    return NULL;
  char *icu_locale = wow_icu_locale(locale, locale_length, error);
  if (icu_locale == NULL)
    return NULL;
  UErrorCode status = U_ZERO_ERROR;
  UNumberFormat *formatter =
      unum_open(icu_style, NULL, 0, icu_locale, NULL, &status);
  free(icu_locale);
  if (U_FAILURE(status) || formatter == NULL) {
    if (formatter != NULL)
      unum_close(formatter);
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open number formatter");
    return NULL;
  }
  if (style == WOW_INTEGER) {
    unum_setAttribute(formatter, UNUM_MAX_FRACTION_DIGITS, 0);
    unum_setAttribute(formatter, UNUM_MIN_FRACTION_DIGITS, 0);
    unum_setAttribute(formatter, UNUM_ROUNDING_MODE, UNUM_ROUND_HALFEVEN);
    unum_setAttribute(formatter, UNUM_PARSE_INT_ONLY, 1);
  }
  return formatter;
}

static UBool set_currency(UNumberFormat *formatter, const char *currency,
                          WowIcuError *error) {
  if (currency == NULL)
    return 1;
  UChar code[4] = {0, 0, 0, 0};
  for (int32_t i = 0; i < 3; ++i)
    code[i] = (UChar)(uint8_t)currency[i];
  UErrorCode status = U_ZERO_ERROR;
  unum_setTextAttribute(formatter, UNUM_CURRENCY_CODE, code, 3, &status);
  if (U_FAILURE(status)) {
    wow_icu_fail(error, status, "set currency code");
    return 0;
  }
  return 1;
}

static UBool format_value(UNumberFormat *formatter, double value,
                          WowIcuString *output, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required =
      unum_formatDouble(formatter, value, NULL, 0, NULL, &status);
  if (!wow_icu_preflight(status, error, "size formatted number"))
    return 0;
  UChar *text = wow_icu_allocate(required, sizeof(UChar), error);
  if (text == NULL)
    return 0;
  status = U_ZERO_ERROR;
  int32_t length =
      unum_formatDouble(formatter, value, text, required + 1, NULL, &status);
  if (U_FAILURE(status)) {
    free(text);
    wow_icu_fail(error, status, "format number");
    return 0;
  }
  UBool converted = wow_icu_to_utf8(text, length, output, error);
  free(text);
  return converted;
}

int32_t wow_icu_format(const char *locale, int32_t locale_length, int32_t style,
                       double value, const char *currency, WowIcuString *output,
                       WowIcuError *error) {
  output->data = NULL;
  output->length = 0;
  UNumberFormat *formatter =
      open_formatter(locale, locale_length, style, error);
  if (formatter == NULL)
    return WOW_ICU_ERROR;
  UBool success = set_currency(formatter, currency, error);
  if (success)
    success = format_value(formatter, value, output, error);
  unum_close(formatter);
  return success ? WOW_ICU_OK : WOW_ICU_ERROR;
}

static int32_t parsed_result(UErrorCode status, int32_t consumed,
                             int32_t length, double value, WowIcuError *error) {
  if (status == U_PARSE_ERROR)
    return WOW_ICU_NO_MATCH;
  if (U_FAILURE(status))
    return wow_icu_fail(error, status, "parse number");
  if (consumed == 0 || consumed != length || !isfinite(value))
    return WOW_ICU_NO_MATCH;
  return WOW_ICU_OK;
}

static int32_t parse_value(UNumberFormat *formatter, const UChar *text,
                           int32_t length, int32_t with_currency, double *value,
                           uint8_t currency[4], WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t consumed = 0;
  UChar code[4] = {0, 0, 0, 0};
  *value = with_currency
               ? unum_parseDoubleCurrency(formatter, text, length, &consumed,
                                          code, &status)
               : unum_parseDouble(formatter, text, length, &consumed, &status);
  int32_t result = parsed_result(status, consumed, length, *value, error);
  if (result != WOW_ICU_OK || !with_currency)
    return result;
  for (int32_t i = 0; i < 3; ++i) {
    if (code[i] < 'A' || code[i] > 'Z') {
      return wow_icu_fail(error, U_INTERNAL_PROGRAM_ERROR,
                          "read parsed ISO currency code");
    }
    currency[i] = (uint8_t)code[i];
  }
  currency[3] = 0;
  return WOW_ICU_OK;
}

int32_t wow_icu_parse(const char *locale, int32_t locale_length, int32_t style,
                      const uint8_t *text, int32_t text_length,
                      int32_t with_currency, double *value, uint8_t currency[4],
                      WowIcuError *error) {
  UNumberFormat *formatter =
      open_formatter(locale, locale_length, style, error);
  if (formatter == NULL)
    return WOW_ICU_ERROR;
  int32_t length = 0;
  UChar *input = wow_icu_to_utf16(text, text_length, &length, error);
  int32_t result = WOW_ICU_ERROR;
  if (input != NULL) {
    result = parse_value(formatter, input, length, with_currency, value,
                         currency, error);
    free(input);
  }
  unum_close(formatter);
  return result;
}
