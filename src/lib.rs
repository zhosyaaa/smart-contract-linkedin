use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
};

// Структура для хранения комментариев
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Comment {
    pub author: Pubkey,
    pub content: String,
}

impl Comment {
    pub fn new(author: Pubkey, content: String) -> Self {
        Comment { author, content }
    }
}

// Структура для хранения постов
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Post {
    pub author: Pubkey,
    pub content: String,
    pub comments: Vec<Comment>,
}

impl Post {
    pub fn new(author: Pubkey, content: String) -> Self {
        Post {
            author,
            content,
            comments: Vec::new(),
        }
    }

    pub fn add_comment(&mut self, author: Pubkey, content: String) {
        self.comments.push(Comment::new(author, content));
    }
}

// Структура для профиля пользователя
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserProfile {
    pub is_initialized: bool,
    pub name: String,
    pub bio: String,
    pub profile_picture: String,
    pub address: Pubkey,
    pub friends: HashSet<Pubkey>,
    pub nft_owned: bool,
    pub posts: HashMap<Pubkey, Vec<Post>>,
}

impl UserProfile {
    pub fn new(name: String, bio: String, profile_picture: String, address: Pubkey) -> Self {
        UserProfile {
            is_initialized: true,
            name,
            bio,
            profile_picture,
            address,
            friends: HashSet::new(),
            nft_owned: false,
            posts: HashMap::new(),
        }
    }

    pub fn can_write_post(&self) -> bool {
        self.nft_owned && self.friends.len() >= 5
    }

    pub fn can_comment(&self) -> bool {
        self.nft_owned && self.friends.len() >= 5
    }

    pub fn add_post(&mut self, author: Pubkey, content: String) {
        let post = Post::new(author, content);
        self.posts.entry(author).or_insert_with(Vec::new).push(post);
    }

    pub fn add_comment(
        &mut self,
        post_author: Pubkey,
        post_index: usize,
        comment_author: Pubkey,
        content: String,
    ) -> ProgramResult {
        if let Some(posts) = self.posts.get_mut(&post_author) {
            if let Some(post) = posts.get_mut(post_index) {
                post.add_comment(comment_author, content);
                return Ok(());
            }
        }
        Err(ProgramError::InvalidAccountData)
    }

    pub fn get_post_with_comments(&self, author: &Pubkey, post_index: usize) -> Option<&Post> {
        self.posts.get(author)?.get(post_index)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ProfessionalNetworkingInstruction {
    CreateUserProfile {
        name: String,
        bio: String,
        profile_picture: String,
    },
    SendFriendRequest {
        friend_address: Pubkey,
    },
    AcceptFriendRequest {
        friend_address: Pubkey,
    },
    WritePost {
        content: String,
    },
    AddComment {
        post_author: Pubkey,
        post_index: usize,
        content: String,
    },
}

entrypoint!(process_instruction);

fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ProfessionalNetworkingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;

    match instruction {
        ProfessionalNetworkingInstruction::CreateUserProfile {
            name,
            bio,
            profile_picture,
        } => {
            let mut user_data = user_account.try_borrow_mut_data()?;
            let new_user_profile = UserProfile::new(name, bio, profile_picture, *user_account.key);
            let serialized_data = new_user_profile.try_to_vec()?;
            user_data[..serialized_data.len()].copy_from_slice(&serialized_data);
            Ok(())
        }

        ProfessionalNetworkingInstruction::SendFriendRequest { friend_address } => {
            let mut user_data = user_account.try_borrow_mut_data()?;
            let mut user_profile = UserProfile::try_from_slice(&user_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            if user_profile.friends.contains(&friend_address) {
                return Err(ProgramError::InvalidAccountData);
            }

            user_profile.friends.insert(friend_address);
            let serialized_data = user_profile.try_to_vec()?;
            user_data[..serialized_data.len()].copy_from_slice(&serialized_data);

            Ok(())
        }
        ProfessionalNetworkingInstruction::AcceptFriendRequest { friend_address } => {
            let mut user_data = user_account.try_borrow_mut_data()?;
            let mut user_profile = UserProfile::try_from_slice(&user_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            user_profile.friends.insert(friend_address);
            if user_profile.friends.len() >= 5 && !user_profile.nft_owned {
                let nft_mint_account = next_account_info(account_info_iter)?;
                let nft_account = next_account_info(account_info_iter)?;
                let system_program = next_account_info(account_info_iter)?;
                let token_program = next_account_info(account_info_iter)?;
                let rent_sysvar = next_account_info(account_info_iter)?;

                create_nft(
                    nft_mint_account,
                    nft_account,
                    user_account,
                    system_program,
                    token_program,
                    rent_sysvar,
                )?;

                user_profile.nft_owned = true;
            }
            let serialized_data = user_profile.try_to_vec()?;
            user_data[..serialized_data.len()].copy_from_slice(&serialized_data);

            let friend_account = next_account_info(account_info_iter)?;
            let mut friend_data = friend_account.try_borrow_mut_data()?;
            let mut friend_profile = UserProfile::try_from_slice(&friend_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            friend_profile.friends.insert(*user_account.key);
            let serialized_data = friend_profile.try_to_vec()?;
            friend_data[..serialized_data.len()].copy_from_slice(&serialized_data);

            Ok(())
        }

        ProfessionalNetworkingInstruction::WritePost { content } => {
            let mut user_data = user_account.try_borrow_mut_data()?;
            let mut user_profile = UserProfile::try_from_slice(&user_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            if !user_profile.can_write_post() {
                return Err(ProgramError::InvalidAccountData);
            }

            user_profile.add_post(*user_account.key, content);
            let serialized_data = user_profile.try_to_vec()?;
            user_data[..serialized_data.len()].copy_from_slice(&serialized_data);

            Ok(())
        }

        ProfessionalNetworkingInstruction::AddComment {
            post_author,
            post_index,
            content,
        } => {
            let mut user_data = user_account.try_borrow_mut_data()?;
            let mut user_profile = UserProfile::try_from_slice(&user_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            if !user_profile.can_comment() {
                return Err(ProgramError::InvalidAccountData);
            }

            user_profile.add_comment(post_author, post_index, *user_account.key, content);
            user_profile.serialize(&mut &mut user_data[..])?;

            Ok(())
        }
    }
}
fn create_nft<'a>(
    nft_mint_account: &'a AccountInfo<'a>,
    nft_account: &'a AccountInfo<'a>,
    user_account: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    rent_sysvar: &'a AccountInfo<'a>,
) -> ProgramResult {
    let rent = Rent::from_account_info(rent_sysvar)?;
    let nft_mint_key = nft_mint_account.key;
    let user_key = user_account.key;

    let signers_seeds: &[&[_]] = &[&user_key.to_bytes(), &[user_account.lamports() as u8]];

    // Create the mint account
    let mint_ix = solana_program::system_instruction::create_account(
        user_key,
        nft_mint_key,
        rent.minimum_balance(82),
        82,
        &spl_token::id(),
    );
    invoke_signed(
        &mint_ix,
        &[
            user_account.clone(),
            nft_mint_account.clone(),
            system_program.clone(),
        ],
        &[signers_seeds],
    )?;

    // Initialize the mint account
    let init_mint_ix =
        spl_token::instruction::initialize_mint(&spl_token::id(), nft_mint_key, user_key, None, 0)?;
    invoke_signed(
        &init_mint_ix,
        &[
            nft_mint_account.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
        ],
        &[signers_seeds],
    )?;

    // Create the token account for the user
    let create_token_account_ix = solana_program::system_instruction::create_account(
        user_key,
        nft_account.key,
        rent.minimum_balance(165),
        165,
        &spl_token::id(),
    );
    invoke_signed(
        &create_token_account_ix,
        &[
            user_account.clone(),
            nft_account.clone(),
            system_program.clone(),
        ],
        &[signers_seeds],
    )?;

    // Initialize the token account
    let init_token_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        nft_account.key,
        nft_mint_key,
        user_key,
    )?;
    invoke_signed(
        &init_token_account_ix,
        &[
            nft_account.clone(),
            nft_mint_account.clone(),
            user_account.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
        ],
        &[signers_seeds],
    )?;

    // Mint the token to the user's account
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        nft_mint_key,
        nft_account.key,
        user_key,
        &[],
        1,
    )?;
    invoke_signed(
        &mint_to_ix,
        &[
            nft_mint_account.clone(),
            nft_account.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
        &[signers_seeds],
    )?;

    Ok(())
}
