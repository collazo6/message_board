#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Api, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CountResponse, PostResponse, UserResponse};
use crate::state::{State, STATE, USERDATA, POSTDATA, UserData, PostData};

// Version info for migration.
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        post_count: msg.post_count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Post { post_text } => try_post(deps, info, post_text),
        ExecuteMsg::SignUp { username } => try_sign_up(deps, info, username),
        ExecuteMsg::LikePost { post_id } => try_like_post(deps, info, post_id),
        ExecuteMsg::Blacklist { user_addr } => try_blacklist(deps, info, user_addr),
    }
}

// Attempts to make a new post.
pub fn try_post(deps: DepsMut, info: MessageInfo, post_text: String) -> Result<Response, ContractError> {

    // Ensure user is signed up and not blacklisted.
    let user = USERDATA.may_load(deps.storage, &info.sender)?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if user.as_ref() == None {
            return Err(ContractError::NotSignedUp {});
        } else if user.as_ref().unwrap().blacklist == true {
            return Err(ContractError::Blacklisted {});
        }
        state.post_count += 1;
        Ok(state)
    })?;

    // Create post data if user signed up and able.
    let post_data = PostData {
        username: user.unwrap().username,
        post_text,
        user_likes: vec![],
    };
    let state = STATE.load(deps.storage)?;
    POSTDATA.save(deps.storage, &state.post_count.to_string(), &post_data)?;

    Ok(Response::new()
        .add_attribute("method", "try_post")
    )
}

// Attempts to create a new user.
pub fn try_sign_up(deps: DepsMut, info: MessageInfo, username: String) -> Result<Response, ContractError> {
    
    // Ensure user does not already have an account.
    let user = USERDATA.may_load(deps.storage, &info.sender)?;
    if user != None {
        return Err(ContractError::AlreadyHaveAccount {});
    }

    // Create user data and store in map.
    let user_data = UserData {
        username,
        blacklist: false,
    };
    USERDATA.save(deps.storage, &info.sender, &user_data)?;

    Ok(Response::new()
        .add_attribute("method", "try_sign_up")
    )
}

// Attempts to like a post.
pub fn try_like_post(deps: DepsMut, info: MessageInfo, post_id: u64) -> Result<Response, ContractError> {
    
    // Ensure user is valid.
    let user = USERDATA.may_load(deps.storage, &info.sender)?;
    if user.as_ref() == None {
        return Err(ContractError::NotSignedUp {})
    } else if user.as_ref().unwrap().blacklist == true {
        return Err(ContractError::Blacklisted {})
    }
    let valid_user = user.unwrap();

    // Add username to post user likes if post is available.
    POSTDATA.update(deps.storage, &post_id.to_string(), |post_data| -> Result<_, ContractError> {
        if let Some(mut pd) = post_data {
            pd.user_likes.push(valid_user.username);
            return Ok(pd);
        }
        return Err(ContractError::PostNotAvailable {});
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_like_post")
    )
}

// Attempts to blacklist a user.
pub fn try_blacklist(deps: DepsMut, info: MessageInfo, user_addr: Addr) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    // Only owner of contract may blacklist users.
    if info.sender == state.owner {
        USERDATA.update(deps.storage, &user_addr, |user_data| -> Result<_, ContractError> {
            if let Some(mut ud) = user_data {
                ud.blacklist = true;
                return Ok(ud);
            }
            return Err(ContractError::UserUnavailable {});
        })?;
    } else {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("method", "try_blacklist")
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetPost { post_id } => to_binary(&query_post(deps, post_id)?),
        QueryMsg::GetUser { user_addr } => to_binary( &query_user(deps, user_addr)?)
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { post_count: state.post_count })
}

fn query_post(deps: Deps, post_id: u64) -> StdResult<PostResponse> {
    let post_data = POSTDATA.load(deps.storage, &post_id.to_string());
    Ok(PostResponse { post_data: post_data.unwrap() })
}

fn query_user(deps: Deps, user_addr: Addr) -> StdResult<UserResponse> {
    let user_data = USERDATA.load(deps.storage, &user_addr);
    Ok(UserResponse { user_data: user_data.unwrap() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { post_count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // Call .unwrap() to assert initialization was a success.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Ensure post_count is properly initialized.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.post_count);
    }

    #[test]
    fn get_user() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { post_count: 0 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // User may sign up and information may be retrieved.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SignUp { username: "someguy6".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // User may not create multiple accounts.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SignUp { username: "someotherguy3".to_string()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::AlreadyHaveAccount {}) => {}
            _ => panic!("Must return an already have account error!"),
        }

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUser { user_addr: deps.api.addr_validate("anyone").unwrap() }).unwrap();
        let value: UserResponse = from_binary(&res).unwrap();
        let expected_output= UserResponse{ user_data: UserData {
            username: "someguy6".to_string(),
            blacklist: false,
        }};
        assert_eq!(expected_output, value);
    }

    #[test]
    fn post() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { post_count: 0 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // User may create and like a post if signed up and post is available.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SignUp { username: "someguy6".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Post { post_text: "Terra is the best!".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::LikePost { post_id: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPost { post_id: 1 }).unwrap();
        let value: PostResponse = from_binary(&res).unwrap();
        let expected_output= PostResponse{ post_data: PostData {
            username: "someguy6".to_string(),
            post_text: "Terra is the best!".to_string(),
            user_likes: vec!["someguy6".to_string()],
        }};
        assert_eq!(expected_output, value);

        // User may not create or like a post if not signed up.
        let info = mock_info("someone", &coins(2, "token"));
        let msg = ExecuteMsg::Post { post_text: "Price of UST is high. Make sure to sell!".to_string()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NotSignedUp {}) => {}
            _ => panic!("Must return a not signed up error!"),
        }

        let info = mock_info("someone", &coins(2, "token"));
        let msg = ExecuteMsg::LikePost { post_id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NotSignedUp {}) => {}
            _ => panic!("Must return a not signed up error!"),
        }

        // A signed up user may not like an unavailable post
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::LikePost { post_id: 0 };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::PostNotAvailable {}) => {}
            _ => panic!("Must return a post not available error!"),
        }
    }

    #[test]
    fn blacklist() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { post_count: 0 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Any address that isn't the contract owner is unauthorized to blacklist any user.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Blacklist { user_addr: deps.api.addr_validate("creator").unwrap() };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return an unauthorized error!"),
        }

        // Owner cannot blacklist an address that isn't registered.
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Blacklist { user_addr: deps.api.addr_validate("anyone").unwrap()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::UserUnavailable {}) => {}
            _ => panic!("Must return a user unavailable error!"),
        }

        // Registered user may be blacklisted by owner.
        let info = mock_info(">:(", &coins(2, "token"));
        let msg = ExecuteMsg::SignUp { username: "badguy".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Blacklist { user_addr: deps.api.addr_validate(">:(").unwrap() };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUser { user_addr: deps.api.addr_validate(">:(").unwrap() }).unwrap();
        let value: UserResponse = from_binary(&res).unwrap();
        let expected_output= UserResponse{ user_data: UserData {
            username: "badguy".to_string(),
            blacklist: true,
        }};
        assert_eq!(expected_output, value);

        // Blacklisted user may not create or like a post :(.
        let info = mock_info("user", &coins(2, "token"));
        let msg = ExecuteMsg::SignUp { username: "postguy".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("user", &coins(2, "token"));
        let msg = ExecuteMsg::Post { post_text: "First Post!".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(">:(", &coins(2, "token"));
        let msg = ExecuteMsg::LikePost { post_id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::Blacklisted {}) => {}
            _ => panic!("Must return a blacklist error!"),
        }

        let info = mock_info(">:(", &coins(2, "token"));
        let msg = ExecuteMsg::Post { post_text: "I can't post. :(".to_string()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::Blacklisted {}) => {}
            _ => panic!("Must return a blacklist error!"),
        }
    }
}