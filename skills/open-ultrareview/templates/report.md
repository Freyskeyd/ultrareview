# Open Ultrareview Report

**Date:** {date}
**Reviewer model:** {reviewer_model}
**Verifier model:** {verifier_model}
**Base ref:** {base_ref}
**Files reviewed:** {file_count}
**Lines changed:** {line_count}
**Dimensions:** {dimensions}

---

## Summary

| Category | Count |
|----------|-------|
| Confirmed bugs (high confidence) | {confirmed_bugs_high} |
| Confirmed bugs (medium confidence) | {confirmed_bugs_medium} |
| Confirmed improvements (high confidence) | {confirmed_improvements_high} |
| Confirmed improvements (medium confidence) | {confirmed_improvements_medium} |
| Unverified findings | {unverified_count} |
| Rejected findings | {rejected_count} |

---

## Confirmed Findings

### Bugs

{for each confirmed bug finding:}

#### [{dimension}] {title}

- **File:** `{file}:{line}:{col}`
- **Severity:** {severity}
- **Evidence:** {evidence}
- **Suggestion:** {suggestion}
- **Verified by:** {verifier_model} ({confidence} confidence)
- **Verifier reasoning:** {independent_reasoning}

{end for}

### Improvements

{for each confirmed improvement finding:}

#### [{dimension}] {title}

- **File:** `{file}:{line}:{col}`
- **Severity:** {severity}
- **Rationale:** {rationale}
- **Suggestion:** {suggestion}
- **Verified by:** {verifier_model} ({confidence} confidence)
- **Verifier reasoning:** {independent_reasoning}

{end for}

---

## Unverified Findings

These findings could not be independently verified.
They may still be valid and warrant human review.

{for each unverified finding:}

#### [{dimension}] {title}

- **File:** `{file}:{line}:{col}`
- **Severity:** {severity}
- **Reason unverified:** {reason}

{end for}

---

## Rejected Findings

These findings were reviewed and rejected by the verifier.
Included for transparency.

{for each rejected finding:}

#### [{dimension}] {title}

- **File:** `{file}:{line}:{col}`
- **Rejection reasoning:** {independent_reasoning}

{end for}
