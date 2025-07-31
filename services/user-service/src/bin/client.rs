pub mod user {
     tonic::include_proto!("user");
}

use user::user_service_client::UserServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
     let mut client = UserServiceClient::connect("http://[::1]:50051").await?;
     
     let request = tonic::Request::new(user::GetUserRequest {
          id: "test-id-123".to_string(),
     });
     
     let response = client.get_user(request).await?;
     
     println!("Response: {:?}", response.into_inner());

     println!("\n--- Testing create_user ---");

     let create_request = tonic::Request::new(user::CreateUserRequest {
          email: "newuser@example.com".to_string(),
          username: "newuser".to_string(),
          password: "secret123".to_string(),
          role: user::UserRole::Developer as i32,
     });

     let create_response = client.create_user(create_request).await?;

     println!("Created user: {:?}", create_response.into_inner());
     
     Ok(())
}