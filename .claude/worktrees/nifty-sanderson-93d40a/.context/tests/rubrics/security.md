# Scoring Rubric: Security

Measures adherence to security guidelines and absence of vulnerabilities.

## Score Definitions

### 10 - Secure
- All security guidelines followed
- Uses Argon2id correctly
- No sensitive data exposure
- All inputs validated
- Authentication enforced
- Authorization checked

### 9 - Excellent
- All major security requirements met
- One very minor security improvement possible

### 8 - Very Good
- All critical security requirements met
- Minor security enhancement possible

### 7 - Good
- No critical vulnerabilities
- One moderate security concern

### 6 - Acceptable
- No critical vulnerabilities
- Multiple minor security concerns
- Basic protections in place

### 5 - Marginal
- Some security measures present
- One moderate vulnerability
- Missing some validation

### 4 - Below Standard
- Basic security only
- Multiple moderate vulnerabilities
- Authorization incomplete

### 3 - Poor
- Critical vulnerability present
- Missing authentication
- Data exposure risk

### 2 - Very Poor
- Multiple critical vulnerabilities
- No security measures
- Would fail security audit

### 1 - Dangerous
- Severe security flaws
- Passwords exposed
- Data completely unprotected

### 0 - Critical Failure
- Stores plain text passwords
- No authentication whatsoever
- Actively dangerous code

## Critical Security Requirements

These are MANDATORY and failure results in maximum deduction:

| Requirement | Violation Deduction |
|-------------|---------------------|
| Uses Argon2id (NOT bcrypt) | -8 |
| Never returns passwordHash | -10 |
| Authentication on protected routes | -10 |
| Ownership verification | -8 |
| Input validation | -5 |
| No secrets in responses | -10 |

## Security Checklist

- [ ] Uses Argon2id with correct config
- [ ] Authentication middleware applied
- [ ] Authorization/ownership checked
- [ ] Input validated with Zod
- [ ] No sensitive data in responses
- [ ] Proper error codes (no info leakage)
- [ ] No SQL injection possible
- [ ] No secrets logged
