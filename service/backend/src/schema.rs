// @generated automatically by Diesel CLI.

diesel::table! {
    comments (CommentsID) {
        CommentsID -> Integer,
        comment -> Text,
        UserID -> Nullable<Integer>,
        VideoID -> Nullable<Integer>,
    }
}

diesel::table! {
    users (UserID) {
        UserID -> Integer,
        name -> Text,
        password -> Text,
        about -> Nullable<Text>,
    }
}

diesel::table! {
    videos (VideoID) {
        VideoID -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        source -> Binary,
        UserID -> Nullable<Integer>,
    }
}

diesel::joinable!(comments -> users (UserID));
diesel::joinable!(comments -> videos (VideoID));
diesel::joinable!(videos -> users (UserID));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    users,
    videos,
);
