use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use shared::{AppError, Claims};
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct PermissionCheck {
    required_permission: String,
}

impl PermissionCheck {
    pub fn new(required_permission: String) -> Self {
        Self {
            required_permission,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PermissionCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PermissionCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let required_permission = self.required_permission.clone();
        ready(Ok(PermissionCheckMiddleware {
            service: Rc::new(service),
            required_permission,
        }))
    }
}

pub struct PermissionCheckMiddleware<S> {
    service: Rc<S>,
    required_permission: String,
}

impl<S, B> Service<ServiceRequest> for PermissionCheckMiddleware<S>
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
        let required_permission = self.required_permission.clone();

        Box::pin(async move {
            // Get claims from request extensions (set by JwtAuth middleware)
            let has_permission = {
                let extensions = req.extensions();
                let claims = extensions
                    .get::<Claims>()
                    .ok_or_else(|| AppError::Unauthorized("Missing authentication".to_string()))?;
                claims.permissions.contains(&required_permission)
            };

            if !has_permission {
                return Err(AppError::Forbidden(format!(
                    "Permission '{}' required",
                    required_permission
                ))
                .into());
            }

            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
