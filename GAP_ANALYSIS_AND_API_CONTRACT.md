# Gap Analysis & API Contract Report

## 1. Overview
This document analyzes the gap between the existing frontend (`hcp-ui`) API expectations and the current backend gateway (`hcp-gateway`) implementation. It serves as the blueprint for the refactoring process.

## 2. API Contract Checklist

### 2.1 General Standards
| Category | Frontend Expectation | Current Gateway | Gap |
|Data Wrapper| `ApiResponse<T> { code: 0, message: "", data: T }` | Matches | None (Logic check needed) |
|Base URL| `/api/*` (implied by typical VITE setup) | `/*` | **Critical**: Gateway serves at root |
|Auth| Header `Authorization: Bearer <token>` | Ignored | **Critical**: No Auth verification |
|CORS| Specific Origin | Permissive `*` | **Major**: Needs restriction |
|Error Codes| Non-zero `code` in JSON body | `ApiResponse::error` used | **Minor**: Ensure 500 maps to JSON |
|WebSocket| `/performance` (likely) | None | **Critical**: Missing WebSocket handler |

### 2.2 Endpoint Analysis

#### Transaction Module (`/transactions`)
- **Frontend**:
    - `POST /submit` (Body: `TransactionSubmitRequest`)
    - `GET /query` (Params: `status`, `from`, `to`, `limit`, `offset`)
- **Gateway**:
    - `POST /submit` exists.
    - `GET /query` exists but pagination support in `models.rs` / `transaction.rs` needs verification.

#### Performance Module (`/performance`)
- **Frontend**:
    - `GET /metrics`
    - `WS /performance` (WebSocket for real-time updates)
- **Gateway**:
    - `GET /metrics` exists.
    - **WebSocket Missing**.

#### Analysis Module (`/analysis`)
- **Frontend**:
    - `POST /export` (Expects Blob/File)
- **Gateway**:
    - `POST /export` exists, needs to ensure `Content-Disposition` header and correct streaming response.

## 3. Implementation Plan

### 3.1 Dependencies
- Added `jsonwebtoken`, `validator`, `tower-governor`, `bcrypt`.

### 3.2 Core Infrastructure
- **Router**: Move all routes to `/api`.
- **Middleware**:
    - `AuthMiddleware`: Verify JWT.
    - `CorsLayer`: Restrict to UI origin.
    - `RateLimit`: Protect endpoints.
- **Validation**: Add `#[validate]` to request structs.

### 3.3 Module Specifics
- **WebSocket**: Implement `axum::extract::ws::WebSocketUpgrade` in `api/performance.rs`.
- **Transactions**: Ensure `limit`/`offset` works in memory mock.

## 4. Deliverables status
- [x] Gap Analysis
- [ ] Refactored Gateway Code
- [ ] Postman Tests
