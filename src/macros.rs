#[macro_export]
macro_rules! weapon_switcher {
    ($name:ident { $($prev:tt <= $current:ident => $next:tt,)+  }) => {
        enum $name {
            $($current,)+
        }

        impl $name {
            fn prev_weapon(&mut self, entity: Entity, cmd: &mut Commands) {
                match self {
                            $(Self::$current => {
                                cmd.remove_one::<$current>(entity);
                                cmd.insert_one(entity, $prev::default());
                                *self = Self::$prev;
                            })+
                        }
            }

            fn next_weapon(&mut self, entity: Entity, cmd: &mut Commands) {
                match self {
                            $(Self::$current => {
                                cmd.remove_one::<$current>(entity);
                                cmd.insert_one(entity, $next::default());
                                *self = Self::$next;
                            })+
                        }
            }
        }
    };
}