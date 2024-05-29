#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::signature::Keypair;

    #[tokio::test]
    async fn test_create_user_profile() {
        let program_id = Pubkey::new_unique();
        let mut test = ProgramTest::new("professional_networking", program_id, processor!(process_instruction));
        let user_account = Keypair::new();

        test.add_account(user_account.pubkey(), Account::new(0, 0, &program_id));
        let (mut banks_client, payer, recent_blockhash) = test.start().await;
        let instruction_data = ProfessionalNetworkingInstruction::CreateUserProfile {
            name: "Alice".to_string(),
            bio: "Bio of Alice".to_string(),
            profile_picture: "url-to-picture".to_string(),
        }
        .try_to_vec()
        .unwrap();

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &instruction_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        assert_eq!(user_profile.name, "Alice");
        assert_eq!(user_profile.bio, "Bio of Alice");
        assert_eq!(user_profile.profile_picture, "url-to-picture");
    }

    #[tokio::test]
    async fn test_send_friend_request() {
        let program_id = Pubkey::new_unique();
        let mut test = ProgramTest::new("professional_networking", program_id, processor!(process_instruction));
        let user_account = Keypair::new();
        let friend_account = Keypair::new();
        test.add_account(user_account.pubkey(), Account::new(0, 0, &program_id));
        test.add_account(friend_account.pubkey(), Account::new(0, 0, &program_id));

        let (mut banks_client, payer, recent_blockhash) = test.start().await;

        let instruction_data = ProfessionalNetworkingInstruction::SendFriendRequest {
            friend_address: friend_account.pubkey(),
        }
        .try_to_vec()
        .unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &instruction_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &user_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        assert!(user_profile.friends.contains(&friend_account.pubkey()));
    }

    #[tokio::test]
    async fn test_accept_friend_request() {
        let program_id = Pubkey::new_unique();
        let mut test = ProgramTest::new("professional_networking", program_id, processor!(process_instruction));
        let user_account = Keypair::new();
        let friend_account = Keypair::new();
        test.add_account(user_account.pubkey(), Account::new(0, 0, &program_id));
        test.add_account(friend_account.pubkey(), Account::new(0, 0, &program_id));
        let (mut banks_client, payer, recent_blockhash) = test.start().await;

        let send_friend_request_data = ProfessionalNetworkingInstruction::SendFriendRequest {
            friend_address: friend_account.pubkey(),
        }
        .try_to_vec()
        .unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &send_friend_request_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &user_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
        let accept_friend_request_data = ProfessionalNetworkingInstruction::AcceptFriendRequest {
            friend_address: user_account.pubkey(),
        }
        .try_to_vec()
        .unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &accept_friend_request_data, vec![friend_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &friend_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        assert!(user_profile.friends.contains(&friend_account.pubkey()));
    }

    async fn test_write_post() {
        let program_id = Pubkey::new_unique();
        let mut test = ProgramTest::new("professional_networking", program_id, processor!(process_instruction));
        let user_account = Keypair::new();
        test.add_account(user_account.pubkey(), Account::new(0, 0, &program_id));
     
        let (mut banks_client, payer, recent_blockhash) = test.start().await;
        let content = "Hello World!".to_string();
        let write_post_data = ProfessionalNetworkingInstruction::WritePost { content }.try_to_vec().unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &write_post_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &user_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        assert_eq!(user_profile.posts.len(), 1);
        assert_eq!(user_profile.posts.get(&user_account.pubkey()).unwrap().len(), 1);
        assert_eq!(user_profile.posts.get(&user_account.pubkey()).unwrap()[0].content, content);
    }

    #[tokio::test]
    async fn test_add_comment() {
        let program_id = Pubkey::new_unique();
        let mut test = ProgramTest::new("professional_networking", program_id, processor!(process_instruction));
        let user_account = Keypair::new();
        test.add_account(user_account.pubkey(), Account::new(0, 0, &program_id));
        let (mut banks_client, payer, recent_blockhash) = test.start().await;
        let content = "Hello World!".to_string();
        let write_post_data = ProfessionalNetworkingInstruction::WritePost { content }.try_to_vec().unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &write_post_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &user_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        let post_author = user_account.pubkey();
        let post_index = 0; 
        let comment_content = "Nice post!".to_string();
        let add_comment_data = ProfessionalNetworkingInstruction::AddComment {
            post_author,
            post_index,
            content: comment_content.clone(),
        }
        .try_to_vec()
        .unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &add_comment_data, vec![user_account.pubkey()])],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &user_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let user_account_data = banks_client
            .get_account(user_account.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");
        let user_profile = UserProfile::try_from_slice(&user_account_data.data).unwrap();

        let post_with_comments = user_profile.get_post_with_comments(&post_author, post_index).unwrap();
        assert_eq!(post_with_comments.comments.len(), 1);
        assert_eq!(post_with_comments.comments[0].content, comment_content);
    }

}
