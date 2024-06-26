use std::env;
use dotenv::dotenv;
use s3_gateway_rs::controller::s3::{delete_object, get_list_objects, get_valid_file_name, request_signed_url, PresignedObject};
use salvo::{conn::tcp::TcpAcceptor, cors::Cors, http::header, hyper::Method, prelude::*};
extern crate serde_json;
use serde::Serialize;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::new().env().init().unwrap();

	let port: String = match env::var("PORT") {
        Ok(value) => value,
        Err(_) => {
			log::info!("Variable `PORT` Not found from enviroment, as default 7878");
			"7878".to_owned()
		}.to_owned()
	};

	let host: String = "0.0.0.0:".to_owned() + &port;
	log::info!("Server Address: {:?}", host.clone());
	let acceptor: TcpAcceptor = TcpListener::new(&host).bind().await;

    // TODO: Add support to allow requests from multiple origin
    let allowed_origin = match env::var("ALLOWED_ORIGIN") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `ALLOWED_ORIGIN` Not found from enviroment");
            "*".to_owned()
        }.to_owned(),
    };
    let _s3_url =  match env::var("S3_URL") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `S3_URL` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _bucket_name =  match env::var("BUCKET_NAME") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `BUCKET_NAME` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _api_key =  match env::var("API_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `API_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _secret_key =  match env::var("SECRET_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `SECRET_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };

	//  Send Device Info
    let cors_handler = Cors::new()
        .allow_origin(&allowed_origin.to_owned())
        .allow_methods(vec![Method::OPTIONS, Method::GET, Method::DELETE])
        .allow_headers(vec![header::ACCESS_CONTROL_REQUEST_METHOD, header::ACCESS_CONTROL_REQUEST_HEADERS, header::AUTHORIZATION])
        .into_handler()
    ;

	let router = Router::new()
        .hoop(cors_handler)
        .push(
			// /api
			Router::with_path("api")
				.options(options_response)
				.get(get_system_info)
                .push(
                    Router::with_path("resources")
						.options(options_response)
                        .get(get_resources_file_container_based)
                        .push(
                            Router::with_path("<**file_name>")
								.options(options_response)
                                .get(get_resource)
                                .delete(delete_resource)
                        )
                )
                .push(
                    Router::with_path("download-url/<**file_name>")
						.options(options_response)
                        .get(get_presigned_url_download_file)
                )
                .push(
					Router::with_path("presigned-url/<client_id>")
						.push(
							Router::with_path("<file_name>")
								.options(options_response)
								.get(get_presigned_url_put_file_container_based)
						)
						.push(
							Router::with_path("<container_id>/<file_name>")
								.options(options_response)
								.get(get_presigned_url_put_file_container_based)
						)
				)
        )
    ;
    log::info!("{:#?}", router);

    Server::new(acceptor).serve(router).await;
}

#[handler]
async fn options_response<'a>(_req: &mut Request, _res: &mut Response) {
	_res.status_code(StatusCode::NO_CONTENT);
}

#[derive(Serialize)]
struct SystemInfoResponse {
	version: String
}

#[handler]
async fn get_system_info<'a>(_req: &mut Request, _res: &mut Response) {
	let version: String = match env::var("VERSION") {
		Ok(value) => value,
		Err(_) => {
			log::info!("Variable `VERSION` Not found from enviroment, as default `1.0.0-dev`");
			"1.0.0-dev".to_owned()
		}.to_owned()
	};

	let system_info_response = SystemInfoResponse {
		version: version.to_string(),
	};

	_res.status_code(StatusCode::OK)
		.render(
			Json(system_info_response)
		)
	;
}

#[derive(Serialize)]
struct ErrorResponse {
	status: u16,
	message: String
}

#[handler]
async fn get_resource<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    let _seconds = _req.query::<u32>("seconds");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::GET, _seconds).await {
            Ok(url) => _res.render(Redirect::permanent(url)),
            Err(error) => {
				log::error!("Interal Server Error: `{:}`", error.to_string());
				let error_response = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
		log::error!("File Name is mandatory");
		let error_response = ErrorResponse {
			status: StatusCode::INTERNAL_SERVER_ERROR.into(),
			message: "File Name is mandatory".to_string()
		};
		_res.render(
			Json(error_response)
		);
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[handler]
async fn delete_resource<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    if _file_name.is_some() {
        match delete_object(_file_name.unwrap()).await {
            Ok(_) => {
                
            },
            Err(error) => {
				log::error!("Interal Server Error: `{:}`", error.to_string());
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
		log::error!("File Name is mandatory");
		let error_response: ErrorResponse = ErrorResponse {
			status: StatusCode::INTERNAL_SERVER_ERROR.into(),
			message: "File Name is mandatory".to_string()
		};
		_res.render(
			Json(error_response)
		);
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[handler]
async fn get_resources_file_container_based<'a>(_req: &mut Request, _res: &mut Response) {
    let _client_id = _req.query::<String>("client_id");
    let _container_id = _req.query::<String>("container_id");
    let _file_name = _req.param::<String>("file_name");
    let _container_type = _req.query::<String>("container_type");
    let _table_name = _req.query::<String>("table_name");
    let _column_name = _req.query::<String>("column_name");
    let _record_id = _req.query::<String>("record_id");
    let _user_id = _req.query::<String>("user_id");
    let _role_id = _req.query::<String>("role_id");
    let _seconds = _req.query::<u32>("seconds");
    let _object_list = get_list_objects(_client_id.to_owned(), _container_id.to_owned(), _container_type.to_owned(), _table_name.to_owned(), _column_name.to_owned(), _record_id.to_owned(), _user_id.to_owned(), _role_id.to_owned()).await;
    match _object_list {
        Ok(_objects) => {
           _res.render(Json(_objects))
        },
        Err(error) => {
			log::error!("Interal Server Error: `{:}`", error.to_string());
			let error_response: ErrorResponse = ErrorResponse {
				status: StatusCode::INTERNAL_SERVER_ERROR.into(),
				message: error.to_string()
			};
			_res.render(
				Json(error_response)
			);
            _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[handler]
async fn get_presigned_url_put_file_container_based<'a>(_req: &mut Request, _res: &mut Response) {
    let _client_id = _req.param::<String>("client_id");
    let _container_id = _req.param::<String>("container_id");
    let _file_name = _req.param::<String>("file_name");
    let _container_type = _req.query::<String>("container_type");
    let _table_name = _req.query::<String>("table_name");
    let _column_name = _req.query::<String>("column_name");
    let _record_id = _req.query::<String>("record_id");
    let _user_id = _req.query::<String>("user_id");
    let _role_id = _req.query::<String>("role_id");
    let _seconds = _req.query::<u32>("seconds");
    let _ = get_list_objects(_client_id.to_owned(), _container_id.to_owned(), _container_type.to_owned(), _table_name.to_owned(), _column_name.to_owned(), _record_id.to_owned(), _user_id.to_owned(), _role_id.to_owned()).await;
    //  Get Valid File Name
    let _file_name_to_store = get_valid_file_name(_client_id, _container_id, _file_name, _container_type, _table_name, _column_name, _record_id, _user_id, _role_id);
    match _file_name_to_store {
        Ok(_valid_file_name) => {
            match request_signed_url(_valid_file_name.to_owned(), http::Method::PUT, _seconds).await {
                Ok(url) => _res.render(Json(PresignedObject {
                    url: Some(url),
                    file_name: Some(_valid_file_name)
                })),
                Err(error) => {
					log::error!("Interal Server Error: `{:}`", error.to_string());
					let error_response: ErrorResponse = ErrorResponse {
						status: StatusCode::INTERNAL_SERVER_ERROR.into(),
						message: error.to_string()
					};
					_res.render(
						Json(error_response)
					);
                    _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        },
        Err(error) => {
			log::error!("Interal Server Error: `{:}`", error.to_string());
			let error_response: ErrorResponse = ErrorResponse {
				status: StatusCode::INTERNAL_SERVER_ERROR.into(),
				message: error.to_string()
			};
			_res.render(
				Json(error_response)
			);
            _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[handler]
async fn get_presigned_url_download_file<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    let _seconds = _req.query::<u32>("seconds");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::GET, _seconds).await {
            Ok(url) => _res.render(Json(url)),
            Err(error) => {
				log::error!("Interal Server Error: `{:}`", error.to_string());
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
		log::error!("File Name is mandatory");
		let error_response: ErrorResponse = ErrorResponse {
			status: StatusCode::INTERNAL_SERVER_ERROR.into(),
			message: "File Name is mandatory".to_string()
		};
		_res.render(
			Json(error_response)
		);
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}
