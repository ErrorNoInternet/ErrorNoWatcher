#[macro_export]
macro_rules! get_entities {
    ($client:ident) => {{
        let ecs = $client.ecs.read();
        ecs.try_query::<(
            Entity,
            &AzaleaPosition,
            &EntityKindComponent,
            &EntityUuid,
            &LookDirection,
            &MinecraftEntityId,
        )>()
        .map(|mut query| {
            query
                .iter(&ecs)
                .map(|(entity, position, kind, uuid, direction, id)| {
                    (
                        Vec3::from(*position),
                        ecs.get::<CustomName>(entity)
                            .and_then(|name| name.as_ref().map(ToString::to_string)),
                        kind.to_string(),
                        uuid.to_string(),
                        Direction::from(direction),
                        id.0,
                        ecs.get::<Owneruuid>(entity).and_then(|owner| owner.0),
                        ecs.get::<Pose>(entity)
                            .map(|pose| *pose as u8)
                            .unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
    }};
}

#[macro_export]
macro_rules! get_players {
    ($client:ident) => {{
        let ecs = $client.ecs.read();
        ecs.try_query_filtered::<(
            Entity,
            &MinecraftEntityId,
            &EntityUuid,
            &EntityKindComponent,
            &AzaleaPosition,
            &LookDirection,
        ), With<Player>>()
            .map(|mut query| {
                query
                    .iter(&ecs)
                    .filter(|(entity, ..)| ecs.get::<Dead>(*entity).is_none())
                    .map(|(entity, id, uuid, kind, position, direction)| {
                        (
                            id.0,
                            uuid.to_string(),
                            kind.to_string(),
                            Vec3::from(*position),
                            Direction::from(direction),
                            ecs.get::<Pose>(entity)
                                .map(|pose| *pose as u8)
                                .unwrap_or_default(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }};
}
