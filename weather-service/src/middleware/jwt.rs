use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use shared::{AppError, Claims};
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct JwtAuth {
    jwt_secret: String,
}

impl JwtAuth {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| {
                    AppError::Unauthorized("Missing Authorization header".to_string())
                })?;

            if !auth_header.starts_with("Bearer ") {
                return Err(AppError::Unauthorized(
                    "Invalid Authorization header format".to_string(),
                )
                .into());
            }

            let token = auth_header.strip_prefix("Bearer ").unwrap();

            // Validate token
            let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
            let validation = Validation::default();

            let token_data = decode::<Claims>(token, &decoding_key, &validation)
                .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

            // Attach claims to request extensions
            req.extensions_mut().insert(token_data.claims);

            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
