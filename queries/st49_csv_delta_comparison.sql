(
  select * from 'data/csv/*SPUD*csv'
  except
  select * from delta_scan('./data/delta/st49')
)
union al
(
  select * from delta_scan('./data/delta/ st49')
  except
  select * from 'data/csv/*SPUD*.csv'
);