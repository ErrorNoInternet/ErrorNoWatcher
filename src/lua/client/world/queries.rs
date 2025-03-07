#[macro_export]
macro_rules! get_entities {
    ($client:ident) => {{
        let mut ecs = $client.ecs.lock();
        ecs.query::<(
            &AzaleaPosition,
            &CustomName,
            &EntityKind,
            &EntityUuid,
            &LookDirection,
            &MinecraftEntityId,
            Option<&Owneruuid>,
            &Pose,
        )>()
        .iter(&ecs)
        .map(
            |(position, custom_name, kind, uuid, direction, id, owner_uuid, pose)| {
                (
                    Vec3::from(position),
                    custom_name.as_ref().map(ToString::to_string),
                    kind.to_string(),
                    uuid.to_string(),
                    Direction::from(direction),
                    id.0,
                    owner_uuid.map(ToOwned::to_owned),
                    *pose as u8,
                )
            },
        )
        .collect::<Vec<_>>()
    }};
}

#[macro_export]
macro_rules! get_players {
    ($client:ident) => {{
        let mut ecs = $client.ecs.lock();
        ecs.query_filtered::<(
            &MinecraftEntityId,
            &EntityUuid,
            &EntityKind,
            &AzaleaPosition,
            &LookDirection,
            &Pose,
        ), (With<Player>, Without<Dead>)>()
            .iter(&ecs)
            .map(|(id, uuid, kind, position, direction, pose)| {
                (
                    id.0,
                    uuid.to_string(),
                    kind.to_string(),
                    Vec3::from(position),
                    Direction::from(direction),
                    *pose as u8,
                )
            })
            .collect::<Vec<_>>()
    }};
}
